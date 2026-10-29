//! Atomic deterministic gameplay transitions.
use super::content::Reward;
use super::state::Receipt;
use super::{
    Action, Applied, Command, Content, Context, Event, Outcome, RULES_VERSION, Rejection, State,
    TrainingDefinition, valid_id,
};
use std::collections::BTreeSet;

fn reward(
    state: &mut State,
    quest: &str,
    r: &Reward,
    outcomes: &mut Vec<Outcome>,
) -> Result<(), Rejection> {
    if !state.grants.insert((r.id().into(), quest.into())) {
        return Ok(());
    }
    match r {
        Reward::ExplorerXp { amount, .. } => {
            state.xp = state.xp.checked_add(*amount).ok_or(Rejection::Overflow)?
        }
        Reward::StoryFlag { value, .. } => {
            state.flags.insert(value.clone());
        }
        Reward::Feature { value, .. } => {
            state.features.insert(value.clone());
        }
        Reward::Creature { value, .. } => {
            state.creatures.insert(value.clone());
        }
        Reward::InventoryItem { value, amount, .. } => {
            let n = state.inventory.entry(value.clone()).or_default();
            *n = n.checked_add(*amount).ok_or(Rejection::Overflow)?;
        }
    }
    outcomes.push(Outcome::RewardGranted {
        reward_id: r.id().into(),
        quest_id: quest.into(),
    });
    Ok(())
}
fn quest<'a>(
    content: &'a Content,
    id: &str,
) -> Result<(&'a str, &'a [String], &'a [Reward]), Rejection> {
    if let Some(b) = content.beats.get(id) {
        return Ok((&b.chapter_id, &b.requires_beat_ids, &b.rewards));
    }
    let q = content.help.get(id).ok_or(Rejection::NotFound)?;
    let c = &content.creatures[&q.creature_id];
    Ok((&c.region_id, &q.requires_beat_ids, &q.rewards))
}
pub(super) fn training_eligible(
    state: &State,
    content: &Content,
    t: &TrainingDefinition,
) -> Result<(), Rejection> {
    if !state.creatures.contains(&t.creature_id)
        || content.creatures[&t.creature_id].region_id != state.region
        || state.character_level(content, &t.creature_id) < t.required_level
    {
        return Err(Rejection::Locked);
    }
    Ok(())
}
fn eligible(state: &State, content: &Content, id: &str) -> Result<(), Rejection> {
    if let Some(t) = content.training(id) {
        return training_eligible(state, content, t);
    }
    let (region, requires, _) = quest(content, id)?;
    if region != state.region || requires.iter().any(|r| !state.completed.contains(r)) {
        return Err(Rejection::Locked);
    }
    if let Some(b) = content.beats.get(id)
        && (b.finale_steps.iter().any(|r| !state.completed.contains(r))
            || b.required_creature_ids
                .iter()
                .any(|r| !state.creatures.contains(r)))
    {
        return Err(Rejection::Locked);
    }
    Ok(())
}

/// Pure transition; errors leave the input untouched. Duplicate retries return
/// the original receipt before revision checking, even if the current revision
/// has advanced. Same ID with any different body is always a conflict.
pub fn apply(
    state: &State,
    command: &Command,
    content: &Content,
    context: &Context,
) -> Result<Applied, Rejection> {
    if !valid_id(&command.command_id) {
        return Err(Rejection::InvalidCommand);
    }
    if let Some(receipt) = state.receipts.get(&command.command_id) {
        return if receipt.command == *command {
            Ok(Applied {
                state: state.clone(),
                events: receipt.events.clone(),
                duplicate: true,
            })
        } else {
            Err(Rejection::CommandIdReused)
        };
    }
    if command.expected_revision != state.revision {
        return Err(Rejection::RevisionConflict);
    }
    if state.content_version != content.version {
        return Err(Rejection::InvalidContent("state/content mismatch".into()));
    }
    let mut next = state.clone();
    let mut outcomes = Vec::new();
    match &command.action {
        Action::BeginQuest {
            quest_id,
            encounter_id,
        } => {
            eligible(state, content, quest_id)?;
            if state.completed.contains(quest_id) {
                return Err(Rejection::AlreadyCompleted);
            }
            if !valid_id(encounter_id)
                || !context.facts.reachable_quests.contains(quest_id)
                || context.facts.issued_encounters.get(encounter_id) != Some(quest_id)
                || state.pending.contains_key(quest_id)
                || state.pending.values().any(|e| e == encounter_id)
            {
                return Err(Rejection::UnverifiedEncounter);
            }
            if let Some(t) = content.training(quest_id)
                && state.pending.keys().any(|id| {
                    content
                        .training(id)
                        .is_some_and(|other| other.creature_id == t.creature_id)
                })
            {
                return Err(Rejection::Locked);
            }
            next.pending.insert(quest_id.clone(), encounter_id.clone());
            outcomes.push(Outcome::QuestStarted {
                quest_id: quest_id.clone(),
                encounter_id: encounter_id.clone(),
            });
        }
        Action::CompleteQuest {
            quest_id,
            encounter_id,
        } => {
            eligible(state, content, quest_id)?;
            if state.completed.contains(quest_id) {
                return Err(Rejection::AlreadyCompleted);
            }
            if state.pending.get(quest_id) != Some(encounter_id) {
                return Err(Rejection::UnverifiedEncounter);
            }
            let verified = context
                .facts
                .completed_encounters
                .get(encounter_id)
                .filter(|v| v.quest_id == *quest_id)
                .ok_or(Rejection::UnverifiedEncounter)?;
            if let Some(t) = content.training(quest_id) {
                next.pending.remove(quest_id);
                let amount = if next.training_grants.insert(t.id.clone()) {
                    t.xp
                } else {
                    0
                };
                let xp = next.character_xp.entry(t.creature_id.clone()).or_default();
                *xp = xp.checked_add(amount).ok_or(Rejection::Overflow)?;
                if amount > 0 {
                    outcomes.push(Outcome::RewardGranted {
                        reward_id: format!("character-xp:{}", t.id),
                        quest_id: t.id.clone(),
                    });
                }
                outcomes.push(Outcome::TrainingCompleted {
                    training_id: t.id.clone(),
                    creature_id: t.creature_id.clone(),
                    supported: verified.supported,
                    xp_granted: amount,
                    level: next.character_level(content, &t.creature_id),
                });
            } else {
                next.completed.insert(quest_id.clone());
                next.pending.remove(quest_id);
                outcomes.push(Outcome::QuestCompleted {
                    quest_id: quest_id.clone(),
                    supported: verified.supported,
                });
                let (_, _, rewards) = quest(content, quest_id)?;
                for r in rewards {
                    reward(&mut next, quest_id, r, &mut outcomes)?;
                }
                if let Some(b) = content.beats.get(quest_id) {
                    next.flags.extend(b.completion_flags.clone());
                }
                if let Some(q) = content.help.get(quest_id) {
                    next.friendship.insert(q.creature_id.clone(), 1);
                    next.helping_moments
                        .insert((q.creature_id.clone(), quest_id.clone()));
                    next.discoveries.retain(|_, v| v != &q.creature_id);
                    outcomes.push(Outcome::FriendshipIncreased {
                        creature_id: q.creature_id.clone(),
                        points: 1,
                    });
                }
            }
        }
        Action::AbandonQuest {
            quest_id,
            encounter_id,
        } => {
            if state.pending.get(quest_id) != Some(encounter_id) {
                return Err(Rejection::UnverifiedEncounter);
            }
            next.pending.remove(quest_id);
            outcomes.push(Outcome::QuestAbandoned {
                quest_id: quest_id.clone(),
                encounter_id: encounter_id.clone(),
            });
        }
        Action::Travel { region_id } => {
            let chapter = content.chapters.get(region_id).ok_or(Rejection::NotFound)?;
            let entry = &content.beats[&chapter.entry_beat_id];
            if entry
                .requires_beat_ids
                .iter()
                .any(|id| !state.completed.contains(id))
            {
                return Err(Rejection::Locked);
            }
            next.region = region_id.clone();
            outcomes.push(Outcome::Travelled {
                region_id: region_id.clone(),
            });
        }
        Action::Discover => {
            let selected = if let Some(id) = state.discoveries.get(&state.region) {
                id.clone()
            } else {
                let candidates: Vec<_> = content
                    .creatures
                    .values()
                    .filter(|c| {
                        c.region_id == state.region
                            && state.completed.contains(&c.discovery.eligible_after_beat)
                    })
                    .collect();
                if candidates.is_empty() {
                    return Err(Rejection::Locked);
                }
                let missing: Vec<_> = candidates
                    .iter()
                    .filter(|c| !state.creatures.contains(&c.id))
                    .copied()
                    .collect();
                let pool = if missing.is_empty() {
                    let alternate: Vec<_> = candidates
                        .iter()
                        .filter(|c| state.last_discovery.get(&state.region) != Some(&c.id))
                        .copied()
                        .collect();
                    if alternate.is_empty() {
                        candidates
                    } else {
                        alternate
                    }
                } else {
                    missing
                };
                if let Some(first) = pool
                    .iter()
                    .find(|c| c.discovery.guaranteed_first && !state.creatures.contains(&c.id))
                {
                    first.id.clone()
                } else {
                    pool[context.recorded_random_choice as usize % pool.len()]
                        .id
                        .clone()
                }
            };
            next.discoveries
                .insert(state.region.clone(), selected.clone());
            next.last_discovery
                .insert(state.region.clone(), selected.clone());
            outcomes.push(Outcome::CreatureOffered {
                creature_id: selected,
            });
        }
        Action::SelectTeam { creature_ids } => {
            let slots = usize::from(state.features.contains("team_slot_1"))
                + usize::from(state.features.contains("team_slot_2"));
            if creature_ids.len() > slots
                || creature_ids.iter().collect::<BTreeSet<_>>().len() != creature_ids.len()
                || creature_ids.iter().any(|id| !state.creatures.contains(id))
            {
                return Err(Rejection::Locked);
            }
            next.team = creature_ids.clone();
            outcomes.push(Outcome::TeamSelected {
                creature_ids: creature_ids.clone(),
            });
        }
        Action::HelpFriend {
            creature_id,
            completed_quest_id,
        } => {
            let c = content
                .creatures
                .get(creature_id)
                .ok_or(Rejection::NotFound)?;
            let b = content
                .beats
                .get(completed_quest_id)
                .ok_or(Rejection::NotFound)?;
            let moment = (creature_id.clone(), completed_quest_id.clone());
            if !state.creatures.contains(creature_id)
                || !state.completed.contains(completed_quest_id)
                || b.chapter_id != c.region_id
                || state.region != c.region_id
                || state.helping_moments.contains(&moment)
            {
                return Err(Rejection::Locked);
            }
            let points = state
                .friendship(creature_id)
                .saturating_add(1)
                .min(content.max_friendship);
            next.friendship.insert(creature_id.clone(), points);
            next.helping_moments.insert(moment);
            outcomes.push(Outcome::FriendshipIncreased {
                creature_id: creature_id.clone(),
                points,
            });
        }
        Action::Transform {
            creature_id,
            enabled,
        } => {
            let c = content
                .creatures
                .get(creature_id)
                .ok_or(Rejection::NotFound)?;
            if !state.creatures.contains(creature_id)
                || (*enabled
                    && (state.friendship(creature_id) < c.transformation.minimum_friendship
                        || !state.flags.contains(&c.transformation.required_story_flag)))
            {
                return Err(Rejection::Locked);
            }
            if *enabled {
                next.transformed.insert(c.transformation.id.clone());
            } else {
                next.transformed.remove(&c.transformation.id);
            }
            outcomes.push(Outcome::TransformationSet {
                creature_id: creature_id.clone(),
                enabled: *enabled,
            });
        }
        Action::UseItem { item_id, amount } => {
            if *amount == 0 {
                return Err(Rejection::InvalidCommand);
            }
            let n = next
                .inventory
                .get_mut(item_id)
                .ok_or(Rejection::InsufficientInventory)?;
            *n = n
                .checked_sub(*amount)
                .ok_or(Rejection::InsufficientInventory)?;
            outcomes.push(Outcome::ItemUsed {
                item_id: item_id.clone(),
                amount: *amount,
            });
        }
    }
    next.revision = next.revision.checked_add(1).ok_or(Rejection::Overflow)?;
    let mut events = Vec::new();
    for outcome in outcomes {
        next.sequence = next.sequence.checked_add(1).ok_or(Rejection::Overflow)?;
        events.push(Event {
            sequence: next.sequence,
            revision: next.revision,
            command_id: command.command_id.clone(),
            rules_version: RULES_VERSION.into(),
            content_version: content.version.clone(),
            unix_millis: context.unix_millis,
            recorded_random_choice: context.recorded_random_choice,
            outcome,
        });
    }
    next.receipts.insert(
        command.command_id.clone(),
        Receipt {
            command: command.clone(),
            events: events.clone(),
        },
    );
    Ok(Applied {
        state: next,
        events,
        duplicate: false,
    })
}
