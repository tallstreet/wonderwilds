//! Version-pinned deterministic campaign authority. Ownership is supplied by the API.
mod exploration;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignDefinition {
    pub id: String,
    pub version: String,
    pub rules_version: String,
    pub start_location: String,
    pub starters: Vec<String>,
    pub locations: Vec<LocationDefinition>,
    pub companions: Vec<CompanionDefinition>,
    pub actions: Vec<ActionDefinition>,
    pub encounters: Vec<EncounterDefinition>,
    #[serde(default)]
    pub initial_supplies: BTreeMap<String, u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationDefinition {
    pub id: String,
    pub name: String,
    pub camp: bool,
    #[serde(default)]
    pub routes: Vec<RouteDefinition>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteDefinition {
    pub to: String,
    #[serde(default)]
    pub requires_flags: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionDefinition {
    pub id: String,
    pub name: String,
    pub affinity: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialogue {
    pub speaker: String,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuzzleEffect {
    pub id: String,
    pub operation: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub goal: Option<BTreeMap<String, String>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionDefinition {
    pub id: String,
    pub label: String,
    pub location: String,
    #[serde(default)]
    pub requires_flags: Vec<String>,
    #[serde(default)]
    pub forbids_flags: Vec<String>,
    #[serde(default)]
    pub costs: BTreeMap<String, u32>,
    #[serde(default)]
    pub rewards: BTreeMap<String, u32>,
    #[serde(default)]
    pub grants_flags: Vec<String>,
    #[serde(default)]
    pub encounter: Option<String>,
    #[serde(default)]
    pub recruit: Option<String>,
    #[serde(default)]
    pub dialogue: Vec<Dialogue>,
    #[serde(default)]
    pub mission: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub grant_id: String,
    #[serde(default)]
    pub learning_context: Option<String>,
    #[serde(default)]
    pub learning: bool,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub puzzle: Option<PuzzleEffect>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub milestone: Option<String>,
    #[serde(default)]
    pub field_ability: Option<String>,
    #[serde(default)]
    pub field_method: Option<String>,
    #[serde(default)]
    pub discovery_for: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterDefinition {
    pub id: String,
    pub target: u32,
    pub turn_limit: u32,
    pub pressure_limit: u32,
    pub hazards: Vec<u32>,
    pub affinity: String,
    #[serde(default)]
    pub win_flags: Vec<String>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay: Option<RelayDefinition>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelayDefinition {
    pub round_targets: Vec<u32>,
    pub required_affinities: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Phase {
    pub threshold: u32,
    pub affinity: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanionProjection {
    pub id: String,
    pub level: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CampaignState {
    pub campaign_id: String,
    pub content_version: String,
    pub rules_version: String,
    pub location: String,
    pub flags: BTreeSet<String>,
    pub items: BTreeMap<String, u32>,
    pub team: Vec<String>,
    pub energy: BTreeMap<String, u32>,
    pub grants: BTreeSet<String>,
    pub starter: Option<String>,
    pub encounter: Option<EncounterState>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct EncounterState {
    pub id: String,
    pub progress: u32,
    pub pressure: u32,
    pub turns: u32,
    pub tool_charges: u32,
    pub partner_bonus: u32,
    pub bonus_source: Option<String>,
    pub replay: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CampaignCommand {
    ChooseStarter {
        companion_id: String,
    },
    Travel {
        location_id: String,
    },
    Action {
        action_id: String,
    },
    SetTeam {
        companion_ids: Vec<String>,
    },
    Camp,
    Care {
        companion_id: String,
        kind: String,
    },
    Craft {
        recipe: String,
    },
    EncounterTurn {
        companion_id: String,
        movement: String,
    },
    Replay {
        encounter_id: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignOutcome {
    pub state: CampaignState,
    pub recruit: Vec<String>,
    pub grants: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CampaignError {
    VersionMismatch,
    InvalidContent(String),
    IllegalAction,
    LearningRequired,
    InsufficientResources,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignSnapshot {
    pub state: CampaignState,
    pub legal_actions: Vec<ActionDefinition>,
    pub travel: Vec<String>,
    pub companions: Vec<CompanionProjection>,
    pub encounter_definition: Option<EncounterDefinition>,
}
fn has(flags: &BTreeSet<String>, required: &[String]) -> bool {
    required.iter().all(|x| flags.contains(x))
}
fn maximum(level: u8) -> u32 {
    match level {
        0 | 1 => 3,
        2 | 3 => 4,
        _ => 5,
    }
}
fn level(owned: &[CompanionProjection], id: &str) -> u8 {
    owned.iter().find(|c| c.id == id).map_or(1, |c| c.level)
}
fn check(def: &CampaignDefinition, state: &CampaignState) -> Result<(), CampaignError> {
    if state.campaign_id != def.id
        || state.content_version != def.version
        || state.rules_version != def.rules_version
    {
        Err(CampaignError::VersionMismatch)
    } else {
        let ids: BTreeSet<_> = state.team.iter().collect();
        if !def.locations.iter().any(|l| l.id == state.location)
            || state.team.len() > 2
            || ids.len() != state.team.len()
            || state
                .team
                .iter()
                .any(|id| !def.companions.iter().any(|c| &c.id == id))
            || state
                .starter
                .as_ref()
                .is_some_and(|id| !def.starters.contains(id))
            || state
                .energy
                .iter()
                .any(|(id, v)| *v > 5 || !def.companions.iter().any(|c| &c.id == id))
            || state.flags.len() > 4096
            || state.grants.len() > 4096
            || state.items.values().any(|v| *v > 100_000)
            || state.encounter.as_ref().is_some_and(|e| {
                def.encounters
                    .iter()
                    .find(|b| b.id == e.id)
                    .is_none_or(|b| {
                        e.progress >= b.target
                            || e.pressure >= b.pressure_limit
                            || e.turns >= b.turn_limit
                            || e.tool_charges > 3
                            || e.partner_bonus > 3
                    })
            })
        {
            return Err(CampaignError::InvalidContent(
                "invalid campaign checkpoint".into(),
            ));
        }
        Ok(())
    }
}
pub fn adopt(def: &CampaignDefinition) -> CampaignState {
    CampaignState {
        campaign_id: def.id.clone(),
        content_version: def.version.clone(),
        rules_version: def.rules_version.clone(),
        location: def.start_location.clone(),
        flags: BTreeSet::new(),
        items: def.initial_supplies.clone(),
        team: vec![],
        energy: BTreeMap::new(),
        grants: BTreeSet::new(),
        starter: None,
        encounter: None,
    }
}
fn legal(s: &CampaignState, a: &ActionDefinition) -> bool {
    let replay = a
        .encounter
        .as_ref()
        .is_some_and(|id| s.grants.contains(&format!("win:{id}")));
    let recipient = a.recipient.as_ref().or_else(|| s.team.first());
    let earned = a.recipient.is_some()
        && a.milestone.as_ref().is_some_and(|m| {
            recipient.is_some_and(|id| s.flags.contains(&format!("milestone:{id}:{m}")))
        });
    let milestone_ready = a.milestone.as_ref().is_none_or(|m| {
        let (unlock, prior) = match m.as_str() {
            "first_help" => ("welcome_complete", None),
            "field_partnership" => ("echo_complete", Some("first_help")),
            "personal_project" => ("current_complete", Some("field_partnership")),
            "wayfinder" => ("beacon_complete", Some("personal_project")),
            _ => return false,
        };
        s.flags.contains(unlock)
            && prior.is_none_or(|p| {
                recipient.is_some_and(|id| s.flags.contains(&format!("milestone:{id}:{p}")))
            })
    });
    !earned
        && milestone_ready
        && s.encounter.is_none()
        && (s.starter.is_some() || matches!(a.kind.as_str(), "observe" | "practice"))
        && s.location == a.location
        && a.requires_flags.iter().all(|flag| {
            // One completed introduction teaches the controls. Other friends remain optional.
            if a.id == "welcome_depart" && flag.starts_with("practised_") {
                ["practised_pip", "practised_tansy", "practised_nib"]
                    .iter()
                    .any(|practice| s.flags.contains(*practice))
            } else {
                s.flags.contains(flag)
            }
        })
        && (replay || a.forbids_flags.iter().all(|f| !s.flags.contains(f)))
        && (replay || a.puzzle.is_some() || !s.grants.contains(&format!("action:{}", a.id)))
        && (replay
            || a.puzzle.is_some()
            || a.grant_id.is_empty()
            || !s.grants.contains(&a.grant_id))
        && a.costs
            .iter()
            .all(|(k, v)| s.items.get(k).copied().unwrap_or(0) >= *v)
}
fn field_ready(def: &CampaignDefinition, s: &CampaignState, a: &ActionDefinition) -> bool {
    if let Some(relay) = a
        .encounter
        .as_ref()
        .and_then(|id| def.encounters.iter().find(|e| &e.id == id))
        .and_then(|e| e.relay.as_ref())
    {
        let affinities: BTreeSet<_> = s
            .team
            .iter()
            .filter_map(|id| {
                def.companions
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| &c.affinity)
            })
            .collect();
        if affinities.len() < usize::from(relay.required_affinities) {
            return false;
        }
    }
    let Some(ability) = &a.field_ability else {
        return a.field_method.is_none();
    };
    let (seal, affinity, tool) = match ability.as_str() {
        "brace" => ("bridge_seal", "craft", "brace_kit"),
        "bloom" => ("bloom_seal", "growth", "seed_kit"),
        "resonate" => ("echo_seal", "rhythm", "tuning_fork"),
        "ferry" => ("current_seal", "tide", "folding_raft"),
        "illuminate" => ("beacon_seal", "gleam", "lantern"),
        _ => return false,
    };
    s.flags.contains(seal)
        && match a.field_method.as_deref() {
            Some("companion") => s.team.iter().any(|id| {
                def.companions
                    .iter()
                    .any(|c| &c.id == id && c.affinity == affinity)
            }),
            Some("tool") => s.flags.contains(tool),
            _ => false,
        }
}

pub fn snapshot(
    def: &CampaignDefinition,
    s: &CampaignState,
    owned: &[CompanionProjection],
) -> Result<CampaignSnapshot, CampaignError> {
    check(def, s)?;
    Ok(CampaignSnapshot {
        state: s.clone(),
        legal_actions: def
            .actions
            .iter()
            .chain(exploration::finds(def).iter())
            .filter(|a| {
                legal(s, a)
                    && a.discovery_for
                        .as_ref()
                        .is_none_or(|id| !owned.iter().any(|c| &c.id == id))
                    && field_ready(def, s, a)
                    && a.recruit
                        .as_ref()
                        .is_none_or(|id| !owned.iter().any(|c| &c.id == id))
                    && a.recipient
                        .as_ref()
                        .is_none_or(|id| owned.iter().any(|c| &c.id == id))
            })
            .cloned()
            .map(|mut a| {
                if let Some(puzzle) = a.puzzle.as_mut() {
                    puzzle.goal = None;
                }
                if a.encounter
                    .as_ref()
                    .is_some_and(|id| s.grants.contains(&format!("win:{id}")))
                {
                    a.label = format!("Practice again: {}", a.label);
                }
                a
            })
            .collect(),
        travel: if s.encounter.is_some() || s.starter.is_none() {
            vec![]
        } else {
            def.locations
                .iter()
                .find(|l| l.id == s.location)
                .map(|l| {
                    l.routes
                        .iter()
                        .filter(|r| exploration::route_open(s, r))
                        .map(|r| r.to.clone())
                        .collect()
                })
                .unwrap_or_default()
        },
        companions: owned.to_vec(),
        encounter_definition: s
            .encounter
            .as_ref()
            .and_then(|e| def.encounters.iter().find(|d| d.id == e.id).cloned()),
    })
}
fn recover(def: &CampaignDefinition, s: &mut CampaignState, owned: &[CompanionProjection]) {
    s.location = s
        .flags
        .iter()
        .find_map(|f| f.strip_prefix("last_camp:").map(str::to_owned))
        .unwrap_or_else(|| {
            def.locations
                .iter()
                .find(|l| l.camp)
                .map_or_else(|| def.start_location.clone(), |l| l.id.clone())
        });
    for id in &s.team {
        s.energy.insert(id.clone(), maximum(level(owned, id)));
    }
    if let Some(encounter) = &s.encounter {
        s.flags
            .retain(|f| !f.starts_with(&format!("relay:{}:", encounter.id)));
    }
    s.encounter = None;
    s.flags
        .retain(|f| !f.starts_with("free_signature_used:") && !f.starts_with("encounter_reveal:"));
}
fn finish_practice(s: &mut CampaignState) {
    s.team = s
        .flags
        .iter()
        .filter_map(|f| {
            f.strip_prefix("practice_saved_team:")
                .map(|entry| entry.split_once(':').map_or(entry, |(_, id)| id).to_owned())
        })
        .collect();
    s.flags
        .retain(|f| f != "practice_active" && !f.starts_with("practice_saved_team:"));
}
fn forage(s: &mut CampaignState) {
    for (k, v) in [("berry", 2), ("leaf", 2), ("wood", 1), ("glow_pebble", 1)] {
        let n = s.items.entry(k.into()).or_default();
        *n = (*n).max(v);
    }
}
pub fn apply(
    def: &CampaignDefinition,
    state: &CampaignState,
    command: &CampaignCommand,
    owned: &[CompanionProjection],
    verified_learning: bool,
) -> Result<CampaignOutcome, CampaignError> {
    check(def, state)?;
    if let CampaignCommand::EncounterTurn { movement, .. } = command
        && movement == "guided"
    {
        let planned = guided_command(def, state, owned)?;
        return apply(def, state, &planned, owned, verified_learning);
    }
    let mut s = state.clone();
    let mut recruit = vec![];
    let mut grants = vec![];
    if s.encounter.is_some() && !matches!(command, CampaignCommand::EncounterTurn { .. }) {
        return Err(CampaignError::IllegalAction);
    }
    match command {
        CampaignCommand::ChooseStarter { companion_id } => {
            if s.starter.is_some() || !def.starters.contains(companion_id) {
                return Err(CampaignError::IllegalAction);
            }
            s.starter = Some(companion_id.clone());
            s.team = vec![companion_id.clone()];
            s.energy
                .insert(companion_id.clone(), maximum(level(owned, companion_id)));
            s.flags.insert("starter_chosen".into());
            recruit.push(companion_id.clone());
            forage(&mut s);
        }
        CampaignCommand::Travel { location_id } => {
            if s.starter.is_none()
                || !def
                    .locations
                    .iter()
                    .find(|l| l.id == s.location)
                    .is_some_and(|l| {
                        l.routes
                            .iter()
                            .any(|r| &r.to == location_id && exploration::route_open(&s, r))
                    })
            {
                return Err(CampaignError::IllegalAction);
            }
            s.location = location_id.clone();
            if def.locations.iter().any(|l| l.id == s.location && l.camp) {
                s.flags.retain(|f| !f.starts_with("last_camp:"));
                s.flags.insert(format!("last_camp:{}", s.location));
            }
        }
        CampaignCommand::Action { action_id } => {
            if let Some(id) = action_id.strip_prefix("decline:") {
                let offer = def
                    .actions
                    .iter()
                    .find(|a| a.id == id && a.recruit.is_some())
                    .ok_or(CampaignError::IllegalAction)?;
                if !legal(&s, offer) {
                    return Err(CampaignError::IllegalAction);
                }
                s.flags.insert(format!("recruit_declined:{id}"));
                return Ok(CampaignOutcome {
                    state: s,
                    recruit,
                    grants,
                });
            }
            let finds = exploration::finds(def);
            let a = def
                .actions
                .iter()
                .chain(finds.iter())
                .find(|a| &a.id == action_id)
                .ok_or(CampaignError::IllegalAction)?;
            if !legal(&s, a)
                || a.discovery_for
                    .as_ref()
                    .is_some_and(|id| owned.iter().any(|c| &c.id == id))
                || !field_ready(def, &s, a)
                || a.recruit
                    .as_ref()
                    .is_some_and(|id| owned.iter().any(|c| &c.id == id))
                || a.recipient
                    .as_ref()
                    .is_some_and(|id| !owned.iter().any(|c| &c.id == id))
            {
                return Err(CampaignError::IllegalAction);
            }
            if a.learning && !verified_learning {
                return Err(CampaignError::LearningRequired);
            }
            if let Some(puzzle) = &a.puzzle {
                let prefix = format!("puzzle:{}:", puzzle.id);
                match puzzle.operation.as_str() {
                    "set_near" | "set_far" | "set_first" | "set_second" | "set_third" => {
                        let key = puzzle.operation.trim_start_matches("set_");
                        let value = puzzle.value.as_ref().ok_or(CampaignError::IllegalAction)?;
                        let field = format!("{prefix}{key}:");
                        s.flags
                            .retain(|f| !f.starts_with(&field) && f != &format!("{prefix}blocked"));
                        s.flags.insert(format!("{field}{value}"));
                    }
                    "reset" => {
                        s.flags.retain(|f| !f.starts_with(&prefix));
                    }
                    "check" => {
                        let goal = puzzle.goal.as_ref().ok_or(CampaignError::IllegalAction)?;
                        let success = goal.iter().all(|(key, value)| {
                            s.flags.contains(&format!("{prefix}{key}:{value}"))
                        });
                        s.flags.remove(&format!("{prefix}blocked"));
                        if success {
                            let refresh = a
                                .grants_flags
                                .iter()
                                .any(|f| f.ends_with("project_complete") && !s.flags.contains(f));
                            s.flags.insert(format!("{prefix}solved"));
                            s.flags.extend(a.grants_flags.clone());
                            if refresh {
                                forage(&mut s);
                            }
                        } else {
                            s.flags.insert(format!("{prefix}blocked"));
                        }
                    }
                    _ => return Err(CampaignError::IllegalAction),
                }
                return Ok(CampaignOutcome {
                    state: s,
                    recruit,
                    grants,
                });
            }
            if a.kind == "practice" && a.encounter.is_some() {
                let friend =
                    a.id.strip_prefix("practice_")
                        .ok_or(CampaignError::IllegalAction)?;
                if !def.starters.iter().any(|id| id == friend) {
                    return Err(CampaignError::IllegalAction);
                }
                for (index, id) in s.team.iter().enumerate() {
                    s.flags.insert(format!("practice_saved_team:{index}:{id}"));
                }
                s.flags.insert("practice_active".into());
                s.team = vec![friend.into()];
                s.energy.insert(friend.into(), 3);
            }
            if let Some(id) = &a.encounter {
                if !def.encounters.iter().any(|e| &e.id == id) {
                    return Err(CampaignError::IllegalAction);
                }
                s.flags.retain(|f| {
                    !f.starts_with(&format!("relay:{id}:"))
                        && !f.starts_with(&format!("encounter_action:{id}:"))
                });
                s.flags.insert(format!("encounter_action:{id}:{}", a.id));
                s.flags
                    .retain(|flag| !flag.starts_with(&format!("project:{id}:")));
                if id == "trial_bloom" && s.flags.contains("puzzle:garden_water:solved") {
                    s.flags.insert("project:trial_bloom:north:beds".into());
                    s.flags.insert("project:trial_bloom:south:beds".into());
                }
                s.encounter = Some(EncounterState {
                    id: id.clone(),
                    progress: 0,
                    pressure: 0,
                    turns: 0,
                    tool_charges: 3,
                    partner_bonus: 0,
                    bonus_source: None,
                    replay: s.grants.contains(&format!("win:{id}")),
                });
            }
            for (k, v) in &a.costs {
                *s.items.entry(k.clone()).or_default() -= v;
            }
            for (k, v) in &a.rewards {
                let n = s.items.entry(k.clone()).or_default();
                *n = n
                    .checked_add(*v)
                    .ok_or(CampaignError::InsufficientResources)?;
            }
            if let Some(milestone) = &a.milestone {
                let recipient = a
                    .recipient
                    .as_ref()
                    .or_else(|| s.team.first())
                    .ok_or(CampaignError::IllegalAction)?;
                let prior = match milestone.as_str() {
                    "first_help" => None,
                    "field_partnership" => Some("first_help"),
                    "personal_project" => Some("field_partnership"),
                    "wayfinder" => Some("personal_project"),
                    _ => return Err(CampaignError::IllegalAction),
                };
                if prior.is_some_and(|m| !s.flags.contains(&format!("milestone:{recipient}:{m}"))) {
                    return Err(CampaignError::IllegalAction);
                }
                s.flags.insert(format!("milestone:{recipient}:{milestone}"));
            }
            if a.encounter.is_none() {
                s.flags.extend(a.grants_flags.clone());
                s.grants.insert(format!("action:{}", a.id));
            }
            if a.id.starts_with("find_branch_")
                && (1..=3).all(|i| s.flags.contains(&format!("find_branch_{i}")))
            {
                let location = s.location.clone();
                s.location = "bridge_bank".into();
                let mut gathered = apply(
                    def,
                    &s,
                    &CampaignCommand::Action {
                        action_id: "bridge_gather".into(),
                    },
                    owned,
                    false,
                )?;
                gathered.state.location = location;
                return Ok(gathered);
            }
            if a.encounter.is_none() && !a.grant_id.is_empty() {
                s.grants.insert(a.grant_id.clone());
                grants.push(a.grant_id.clone());
            }
            if let Some(id) = &a.recruit {
                recruit.push(id.clone());
            }
            if a.kind == "project"
                || a.grants_flags
                    .iter()
                    .any(|f| f.ends_with("project_complete"))
            {
                forage(&mut s);
            }
        }
        CampaignCommand::SetTeam { companion_ids } => {
            let cap = if s.flags.contains("bridge_seal") {
                2
            } else {
                1
            };
            let unique: BTreeSet<_> = companion_ids.iter().collect();
            if !def.locations.iter().any(|l| l.id == s.location && l.camp)
                || companion_ids.is_empty()
                || companion_ids.len() > cap
                || unique.len() != companion_ids.len()
                || companion_ids
                    .iter()
                    .any(|id| !owned.iter().any(|c| &c.id == id))
            {
                return Err(CampaignError::IllegalAction);
            }
            s.team = companion_ids.clone();
            for id in companion_ids {
                s.energy
                    .entry(id.clone())
                    .or_insert(maximum(level(owned, id)));
            }
        }
        CampaignCommand::Camp => {
            if s.starter.is_none() || !def.locations.iter().any(|l| l.id == s.location && l.camp) {
                return Err(CampaignError::IllegalAction);
            }
            for companion in owned {
                s.energy
                    .insert(companion.id.clone(), maximum(companion.level));
            }
            s.flags.retain(|f| {
                !f.starts_with("free_signature_used:") && !f.starts_with("encounter_reveal:")
            });
        }
        CampaignCommand::Care { companion_id, kind } => {
            if !owned.iter().any(|c| &c.id == companion_id)
                || !def.locations.iter().any(|l| l.id == s.location && l.camp)
                || ![
                    "play",
                    "groom",
                    "quiet_rest",
                    "favourite_place",
                    "specialize_free_signature",
                    "specialize_rally",
                    "appearance_wayfinder",
                    "appearance_base",
                ]
                .contains(&kind.as_str())
            {
                return Err(CampaignError::IllegalAction);
            }
            if kind == "appearance_wayfinder" && level(owned, companion_id) < 5 {
                return Err(CampaignError::IllegalAction);
            }
            if let Some(choice) = kind.strip_prefix("specialize_") {
                if level(owned, companion_id) < 3 {
                    return Err(CampaignError::IllegalAction);
                }
                let prefix = format!("technique:{companion_id}:");
                s.flags.retain(|f| !f.starts_with(&prefix));
                s.flags.insert(format!("{prefix}{choice}"));
            } else {
                s.flags.insert(format!("memory:care:{companion_id}:{kind}"));
            }
        }
        CampaignCommand::Craft { recipe } => {
            if !def.locations.iter().any(|l| l.id == s.location && l.camp) {
                return Err(CampaignError::IllegalAction);
            }
            let costs = match recipe.as_str() {
                "snack" => vec![("berry", 2)],
                "shelter_patch" => vec![("leaf", 2)],
                "trail_marker" => vec![("wood", 1), ("glow_pebble", 1)],
                _ => return Err(CampaignError::IllegalAction),
            };
            if costs
                .iter()
                .any(|(k, v)| s.items.get(*k).copied().unwrap_or(0) < *v)
            {
                return Err(CampaignError::InsufficientResources);
            }
            if ["snack", "shelter_patch", "trail_marker"]
                .iter()
                .map(|k| s.items.get(*k).copied().unwrap_or(0))
                .sum::<u32>()
                >= 3
            {
                return Err(CampaignError::IllegalAction);
            }
            for (k, v) in costs {
                *s.items.entry(k.into()).or_default() -= v;
            }
            *s.items.entry(recipe.clone()).or_default() += 1;
        }
        CampaignCommand::Replay { encounter_id } => {
            let e = def
                .encounters
                .iter()
                .find(|e| &e.id == encounter_id)
                .ok_or(CampaignError::IllegalAction)?;
            if !s.grants.contains(&format!("win:{}", e.id))
                || !def
                    .actions
                    .iter()
                    .any(|a| a.location == s.location && a.encounter.as_ref() == Some(encounter_id))
            {
                return Err(CampaignError::IllegalAction);
            }
            if let Some(relay) = &e.relay {
                let affinities: BTreeSet<_> = s
                    .team
                    .iter()
                    .filter_map(|id| {
                        def.companions
                            .iter()
                            .find(|c| &c.id == id)
                            .map(|c| &c.affinity)
                    })
                    .collect();
                if affinities.len() < usize::from(relay.required_affinities) {
                    return Err(CampaignError::IllegalAction);
                }
            }
            s.flags
                .retain(|f| !f.starts_with(&format!("relay:{}:", e.id)));
            s.flags
                .retain(|flag| !flag.starts_with(&format!("project:{}:", e.id)));
            s.encounter = Some(EncounterState {
                id: e.id.clone(),
                progress: 0,
                pressure: 0,
                turns: 0,
                tool_charges: 3,
                partner_bonus: 0,
                bonus_source: None,
                replay: true,
            });
        }
        CampaignCommand::EncounterTurn {
            companion_id,
            movement,
        } => {
            if movement == "retreat" {
                if s.encounter.is_none() || !s.team.contains(companion_id) {
                    return Err(CampaignError::IllegalAction);
                }
                let practice = s.flags.contains("practice_active");
                recover(def, &mut s, owned);
                if practice {
                    finish_practice(&mut s);
                }
                return Ok(CampaignOutcome {
                    state: s,
                    recruit,
                    grants,
                });
            }
            if !s.team.contains(companion_id) {
                return Err(CampaignError::IllegalAction);
            }
            let mut e = s.encounter.take().ok_or(CampaignError::IllegalAction)?;
            let board = def
                .encounters
                .iter()
                .find(|d| d.id == e.id)
                .ok_or(CampaignError::IllegalAction)?;
            let project = exploration::project_move(&mut s, &mut e, board, movement)?;
            let affinity = board
                .phases
                .iter()
                .rev()
                .find(|p| e.progress >= p.threshold)
                .map_or(board.affinity.as_str(), |p| p.affinity.as_str());
            let friend = def
                .companions
                .iter()
                .find(|c| &c.id == companion_id)
                .ok_or(CampaignError::IllegalAction)?;
            let available = s
                .energy
                .get(companion_id)
                .copied()
                .unwrap_or(maximum(level(owned, companion_id)));
            let (mut cost, mut progress, mut shield) = (0, 0, 0);
            match movement.as_str() {
                _ if project => {
                    shield = u32::MAX;
                }
                "focus" => progress = 1,
                "guard" => {
                    cost = 1;
                    shield = 2;
                }
                "rest" => {
                    s.energy.insert(
                        companion_id.clone(),
                        (available + 2).min(maximum(level(owned, companion_id))),
                    );
                }
                "signature" => {
                    match friend.affinity.as_str() {
                        "gleam" => {
                            cost = 1;
                            progress = 2;
                        }
                        "growth" => {
                            cost = 2;
                            progress = 2;
                            shield = 1;
                        }
                        "craft" => {
                            cost = 1;
                            progress = 1;
                            shield = 1;
                        }
                        "rhythm" => {
                            cost = 1;
                            progress = 1;
                        }
                        "tide" => {
                            cost = 2;
                            progress = 3;
                        }
                        _ => return Err(CampaignError::IllegalAction),
                    }
                    if affinity == friend.affinity {
                        progress += 2;
                    }
                }
                "rally" => {
                    if level(owned, companion_id) < 2 {
                        return Err(CampaignError::IllegalAction);
                    }
                    cost = 1;
                }
                "tool" => {
                    if e.tool_charges == 0 {
                        return Err(CampaignError::InsufficientResources);
                    }
                    e.tool_charges -= 1;
                    progress = 2;
                    shield = 1;
                }
                "snack" | "shelter_patch" | "trail_marker" => {
                    let n = s
                        .items
                        .get_mut(movement)
                        .ok_or(CampaignError::InsufficientResources)?;
                    if *n == 0 {
                        return Err(CampaignError::InsufficientResources);
                    }
                    *n -= 1;
                    match movement.as_str() {
                        "snack" => {
                            s.energy.insert(
                                companion_id.clone(),
                                (available + 2).min(maximum(level(owned, companion_id))),
                            );
                        }
                        "shelter_patch" => shield = 2,
                        _ => {
                            e.partner_bonus = e.partner_bonus.max(2);
                            e.bonus_source = None;
                        }
                    }
                }
                _ => return Err(CampaignError::IllegalAction),
            }
            let free_key = format!("free_signature_used:{companion_id}");
            if movement == "signature"
                && s.flags
                    .contains(&format!("technique:{companion_id}:free_signature"))
                && !s.flags.contains(&free_key)
            {
                cost = 0;
                s.flags.insert(free_key);
            }
            if available < cost {
                return Err(CampaignError::InsufficientResources);
            }
            if movement != "rest" && movement != "snack" {
                s.energy.insert(companion_id.clone(), available - cost);
            }
            if !project && progress > 0 && e.bonus_source.as_ref() != Some(companion_id) {
                progress += e.partner_bonus;
                e.partner_bonus = 0;
                e.bonus_source = None;
            }
            if movement == "rally" {
                e.partner_bonus = if s.flags.contains(&format!("technique:{companion_id}:rally")) {
                    3
                } else {
                    2
                };
                e.bonus_source = Some(companion_id.clone());
            }
            if movement == "signature" && friend.affinity == "rhythm" {
                e.partner_bonus = e.partner_bonus.max(1);
                e.bonus_source = Some(companion_id.clone());
            }
            if let Some(relay) = &board.relay {
                if movement == "signature" && progress > 0 {
                    s.flags
                        .insert(format!("relay:{}:affinity:{}", e.id, friend.affinity));
                }
                let rounds = relay
                    .round_targets
                    .iter()
                    .enumerate()
                    .take_while(|(index, _)| {
                        s.flags
                            .contains(&format!("relay:{}:round:{}", e.id, index + 1))
                    })
                    .count();
                let boundary = *relay
                    .round_targets
                    .get(rounds)
                    .ok_or(CampaignError::IllegalAction)?;
                let affinity_count = s
                    .flags
                    .iter()
                    .filter(|flag| flag.starts_with(&format!("relay:{}:affinity:", e.id)))
                    .count();
                let allowed_boundary = if boundary == board.target
                    && affinity_count < usize::from(relay.required_affinities)
                {
                    boundary - 1
                } else {
                    boundary
                };
                e.progress = (e.progress + progress).min(allowed_boundary);
                if e.progress >= boundary {
                    s.flags
                        .insert(format!("relay:{}:round:{}", e.id, rounds + 1));
                }
            } else {
                e.progress += progress;
            }
            e.turns += 1;
            s.flags
                .retain(|flag| !flag.starts_with("encounter_reveal:"));
            if movement == "signature" && friend.affinity == "gleam" && e.progress < board.target {
                s.flags
                    .insert(format!("encounter_reveal:{}:{}", e.id, e.turns));
            }
            if e.progress >= board.target {
                let practice = s.flags.contains("practice_active");
                if s.grants.insert(format!("win:{}", e.id)) {
                    s.flags.extend(board.win_flags.clone());
                    let bound = s.flags.iter().find_map(|f| {
                        f.strip_prefix(&format!("encounter_action:{}:", e.id))
                            .map(str::to_owned)
                    });
                    for a in def.actions.iter().filter(|a| {
                        a.encounter.as_ref() == Some(&e.id)
                            && bound.as_ref().is_none_or(|id| &a.id == id)
                    }) {
                        s.flags.extend(a.grants_flags.clone());
                        s.grants.insert(format!("action:{}", a.id));
                        if !a.grant_id.is_empty() {
                            s.grants.insert(a.grant_id.clone());
                            grants.push(a.grant_id.clone());
                        }
                    }
                    grants.push(format!("win:{}", e.id));
                    if !practice {
                        forage(&mut s);
                    }
                }
                if practice {
                    finish_practice(&mut s);
                }
            } else {
                e.pressure += board.hazards[(e.turns as usize - 1) % board.hazards.len()]
                    .saturating_sub(shield);
                if e.pressure >= board.pressure_limit || e.turns >= board.turn_limit {
                    let practice = s.flags.contains("practice_active");
                    s.flags
                        .retain(|f| !f.starts_with(&format!("relay:{}:", e.id)));
                    recover(def, &mut s, owned);
                    if practice {
                        finish_practice(&mut s);
                    }
                } else {
                    s.encounter = Some(e);
                }
            }
        }
    }
    Ok(CampaignOutcome {
        state: s,
        recruit,
        grants,
    })
}

/// Find a winning continuation under the visible finite rules, without consuming supplies.
pub fn guided_command(
    def: &CampaignDefinition,
    state: &CampaignState,
    owned: &[CompanionProjection],
) -> Result<CampaignCommand, CampaignError> {
    check(def, state)?;
    let current = state
        .encounter
        .as_ref()
        .ok_or(CampaignError::IllegalAction)?;
    if let Some(movement) = exploration::project_hint(state) {
        return Ok(CampaignCommand::EncounterTurn {
            companion_id: state
                .team
                .first()
                .ok_or(CampaignError::IllegalAction)?
                .clone(),
            movement,
        });
    }
    let win = format!("win:{}", current.id);
    // Search only combat-relevant state. Historical quests, collection receipts and
    // inventory cannot change these supply-free moves, and cloning them per edge
    // made late-game two-partner guidance exceed the HTTP deadline.
    let mut initial = state.clone();
    initial.grants.clear();
    initial.items.clear();
    initial.energy.retain(|id, _| state.team.contains(id));
    initial.flags.retain(|flag| {
        flag.starts_with("technique:")
            || flag.starts_with("free_signature_used:")
            || flag.starts_with(&format!("relay:{}:", current.id))
            || flag.starts_with(&format!("encounter_action:{}:", current.id))
            || flag.starts_with("encounter_reveal:")
            || flag.starts_with("last_camp:")
            || flag == "practice_active"
            || flag.starts_with("practice_saved_team:")
    });
    let key = |s: &CampaignState| {
        let energy: Vec<_> = s
            .team
            .iter()
            .map(|id| {
                s.energy
                    .get(id)
                    .copied()
                    .unwrap_or(maximum(level(owned, id)))
            })
            .collect();
        let flags: Vec<_> = s
            .flags
            .iter()
            .filter(|flag| flag.starts_with("free_signature_used:") || flag.starts_with("relay:"))
            .cloned()
            .collect();
        (
            s.encounter
                .as_ref()
                .expect("only active states are queued")
                .clone(),
            energy,
            flags,
        )
    };
    let mut seen = std::collections::HashSet::from([key(&initial)]);
    // Explore promising continuations first. Breadth-first search spent the
    // entire budget on short, equivalent two-friend preparations before reaching
    // the longer winning sequences on the final optional boards.
    let priority = |s: &CampaignState| {
        let e = s.encounter.as_ref().expect("only active states are queued");
        (
            e.progress,
            std::cmp::Reverse(e.pressure),
            std::cmp::Reverse(e.turns),
        )
    };
    let mut frontier =
        std::collections::BinaryHeap::from([(priority(&initial), std::cmp::Reverse(0usize))]);
    let mut nodes = vec![Some((initial, None))];
    while let Some((_, std::cmp::Reverse(index))) = frontier.pop() {
        let (s, first) = nodes[index]
            .take()
            .expect("each frontier entry is visited once");
        for id in &s.team {
            for movement in ["signature", "tool", "focus", "rest", "guard", "rally"] {
                let command = CampaignCommand::EncounterTurn {
                    companion_id: id.clone(),
                    movement: movement.into(),
                };
                if let Ok(result) = apply(def, &s, &command, owned, false) {
                    let selected = first.clone().or(Some(command));
                    if result.state.encounter.is_none() && result.state.grants.contains(&win) {
                        return selected.ok_or(CampaignError::IllegalAction);
                    }
                    // Deduplicate when enqueueing: the 5,000-state budget bounds
                    // both expansions and resident queue size, without duplicate
                    // full-state branches waiting to be discarded later.
                    if result.state.encounter.is_some()
                        && seen.len() < 5_000
                        && seen.insert(key(&result.state))
                    {
                        frontier.push((priority(&result.state), std::cmp::Reverse(nodes.len())));
                        nodes.push(Some((result.state, selected)));
                    }
                }
            }
        }
    }
    // The same safe recovery command is always available when the visible state has no
    // bounded supply-free continuation. It retains discoveries and consumed items.
    Ok(CampaignCommand::EncounterTurn {
        companion_id: state
            .team
            .first()
            .cloned()
            .ok_or(CampaignError::IllegalAction)?,
        movement: "retreat".into(),
    })
}

pub fn validate(def: &CampaignDefinition) -> Result<(), CampaignError> {
    let fail = |m: &str| CampaignError::InvalidContent(m.into());
    if def.id.is_empty() || def.version.is_empty() || def.rules_version != "campaign-1" {
        return Err(fail("unsupported identity/rules version"));
    }
    let locations: BTreeSet<_> = def.locations.iter().map(|l| &l.id).collect();
    let companions: BTreeSet<_> = def.companions.iter().map(|c| &c.id).collect();
    let encounters: BTreeSet<_> = def.encounters.iter().map(|e| &e.id).collect();
    let actions: BTreeSet<_> = def.actions.iter().map(|a| &a.id).collect();
    if locations.len() != def.locations.len()
        || companions.len() != def.companions.len()
        || encounters.len() != def.encounters.len()
        || actions.len() != def.actions.len()
        || !locations.contains(&def.start_location)
        || !def.locations.iter().any(|l| l.camp)
        || def.starters.len() != 3
        || def.starters.iter().any(|id| !companions.contains(id))
    {
        return Err(fail("duplicate IDs or missing start/camp/starters"));
    }
    for l in &def.locations {
        for r in &l.routes {
            if !locations.contains(&r.to) {
                return Err(fail("unknown route"));
            }
        }
    }
    let mut grants = BTreeSet::new();
    for a in &def.actions {
        if !locations.contains(&a.location)
            || a.recruit
                .as_ref()
                .is_some_and(|id| !companions.contains(id))
            || a.encounter
                .as_ref()
                .is_some_and(|id| !encounters.contains(id))
            || ![
                "observe",
                "gather",
                "craft",
                "learn",
                "choose",
                "field",
                "puzzle",
                "encounter",
                "care",
                "recruit",
                "practice",
                "project",
                "training",
                "exchange",
            ]
            .contains(&a.kind.as_str())
            || (a.encounter.is_some() && (!a.costs.is_empty() || !a.rewards.is_empty()))
            || a.recipient
                .as_ref()
                .is_some_and(|id| !companions.contains(id))
            || a.discovery_for
                .as_ref()
                .is_some_and(|id| !companions.contains(id))
            || a.puzzle.as_ref().is_some_and(|p| {
                !matches!(
                    p.operation.as_str(),
                    "set_near"
                        | "set_far"
                        | "set_first"
                        | "set_second"
                        | "set_third"
                        | "check"
                        | "reset"
                ) || (p.operation == "check" && p.goal.as_ref().is_none_or(BTreeMap::is_empty))
            })
            || a.field_ability.as_ref().is_some_and(|ability| {
                !["brace", "bloom", "resonate", "ferry", "illuminate"].contains(&ability.as_str())
                    || !matches!(a.field_method.as_deref(), Some("companion" | "tool"))
            })
            || (a.field_ability.is_none() && a.field_method.is_some())
            || a.milestone.as_ref().is_some_and(|m| {
                ![
                    "first_help",
                    "field_partnership",
                    "personal_project",
                    "wayfinder",
                ]
                .contains(&m.as_str())
                    || !a.learning
                    || a.kind != "training"
            })
            || a.costs
                .values()
                .chain(a.rewards.values())
                .any(|v| *v > 10_000)
            || (!a.grant_id.is_empty() && !grants.insert(&a.grant_id))
        {
            return Err(fail("invalid action reference or duplicate grant"));
        }
    }
    for e in &def.encounters {
        if e.relay.as_ref().is_some_and(|relay| {
            relay.round_targets.len() != 3
                || relay.round_targets.last() != Some(&e.target)
                || relay.round_targets.first().is_none_or(|n| *n == 0)
                || relay.round_targets.windows(2).any(|p| p[0] >= p[1])
                || !(1..=2).contains(&relay.required_affinities)
        }) || !["gleam", "growth", "craft", "rhythm", "tide"].contains(&e.affinity.as_str())
            || e.phases.iter().any(|p| {
                !["gleam", "growth", "craft", "rhythm", "tide"].contains(&p.affinity.as_str())
                    || p.threshold >= e.target
            })
            || e.hazards.is_empty()
            || e.target == 0
            || e.turn_limit == 0
            || e.pressure_limit == 0
            || e.target > 1000
            || e.turn_limit > 100
            || e.hazards.iter().any(|h| *h > 100)
            || e.phases
                .windows(2)
                .any(|p| p[0].threshold >= p[1].threshold)
        {
            return Err(fail("invalid encounter bounds"));
        }
    }
    for c in &def.companions {
        if !["gleam", "growth", "craft", "rhythm", "tide"].contains(&c.affinity.as_str()) {
            return Err(fail("unknown affinity"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn definition() -> CampaignDefinition {
        let d: CampaignDefinition = serde_json::from_str(include_str!(
            "../../../../../content/campaign/runtime.v1.json"
        ))
        .unwrap();
        validate(&d).unwrap();
        d
    }
    #[test]
    fn one_completed_practice_unlocks_departure_but_other_requirements_remain() {
        let def = definition();
        let departure = def
            .actions
            .iter()
            .find(|a| a.id == "welcome_depart")
            .unwrap();
        for friend in ["pip", "tansy", "nib"] {
            let mut state = adopt(&def);
            state.starter = Some(friend.into());
            state.team = vec![friend.into()];
            state
                .flags
                .extend(["starter_chosen".into(), "welcome_supplies_ready".into()]);
            assert!(!legal(&state, departure));
            state.flags.insert(format!("practised:{friend}"));
            assert!(!legal(&state, departure));
            state.flags.insert(format!("practised_{friend}"));
            assert!(legal(&state, departure));
            let result = apply(
                &def,
                &state,
                &CampaignCommand::Action {
                    action_id: "welcome_depart".into(),
                },
                &owned(friend),
                false,
            )
            .unwrap();
            assert!(result.state.flags.contains("welcome_complete"));
            assert!(!legal(&result.state, departure));
            state.flags.remove("welcome_supplies_ready");
            assert!(!legal(&state, departure));
        }
    }
    #[test]
    fn exploration_finds_are_location_bound_and_award_branches_once() {
        let def = definition();
        let mut s = adopt(&def);
        s.starter = Some("pip".into());
        s.team = vec!["pip".into()];
        s.flags.insert("bridge_clue_complete".into());
        let command = |id: &str| CampaignCommand::Action {
            action_id: id.into(),
        };
        assert!(apply(&def, &s, &command("find_branch_1"), &owned("pip"), false).is_err());
        let view = snapshot(&def, &s, &owned("pip")).unwrap();
        assert!(view.travel.contains(&"picnic_clearing".into()));
        s = apply(
            &def,
            &s,
            &CampaignCommand::Travel {
                location_id: "picnic_clearing".into(),
            },
            &owned("pip"),
            false,
        )
        .unwrap()
        .state;
        for id in ["find_branch_3", "find_branch_1", "find_branch_2"] {
            assert!(
                snapshot(&def, &s, &owned("pip"))
                    .unwrap()
                    .legal_actions
                    .iter()
                    .any(|a| a.id == id)
            );
            s = apply(&def, &s, &command(id), &owned("pip"), false)
                .unwrap()
                .state;
            assert!(apply(&def, &s, &command(id), &owned("pip"), false).is_err());
        }
        assert_eq!(s.items.get("quest_branch"), Some(&3));
        assert_eq!(s.location, "picnic_clearing");
        assert!(s.flags.contains("bridge_gather_done"));
        assert!(s.grants.contains("action:bridge_gather"));
        assert!(
            !snapshot(&def, &s, &owned("pip"))
                .unwrap()
                .legal_actions
                .iter()
                .any(|a| a.id.starts_with("find_branch_"))
        );
    }

    #[test]
    fn construction_and_garden_require_their_own_saved_dependencies() {
        let def = definition();
        for (board_id, wrong, moves) in [
            (
                "trial_bridge",
                "bridge_fit:middle:short",
                vec![
                    "bridge_fit:near:short",
                    "bridge_fit:middle:long",
                    "bridge_fit:far:short",
                    "bridge_brace:near",
                    "bridge_brace:far",
                    "bridge_cross",
                ],
            ),
            (
                "trial_bloom",
                "garden_water:north",
                vec![
                    "garden_route:north:beds",
                    "garden_water:north",
                    "garden_mulch:north",
                    "garden_route:south:beds",
                    "garden_water:south",
                    "garden_mulch:south",
                    "garden_check",
                ],
            ),
        ] {
            let mut s = adopt(&def);
            s.starter = Some("pip".into());
            s.team = vec!["pip".into()];
            s.energy.insert("pip".into(), 0);
            s.encounter = Some(EncounterState {
                id: board_id.into(),
                progress: 0,
                pressure: 0,
                turns: 0,
                tool_charges: 3,
                partner_bonus: 3,
                bonus_source: None,
                replay: false,
            });
            let command = |movement: &str| CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: movement.into(),
            };
            assert!(apply(&def, &s, &command(wrong), &owned("pip"), false).is_err());
            assert!(
                apply(
                    &def,
                    &s,
                    &command(if board_id == "trial_bridge" {
                        "bridge_cross"
                    } else {
                        "garden_check"
                    }),
                    &owned("pip"),
                    false
                )
                .is_err()
            );
            for (i, movement) in moves.iter().enumerate() {
                let result = apply(&def, &s, &command(movement), &owned("pip"), false).unwrap();
                s = serde_json::from_str(&serde_json::to_string(&result.state).unwrap()).unwrap();
                check(&def, &s).unwrap();
                if i + 1 < moves.len() {
                    assert!(s.encounter.is_some());
                    assert_eq!(s.encounter.as_ref().unwrap().pressure, 0);
                }
            }
            assert!(s.encounter.is_none());
            assert!(s.grants.contains(&format!("win:{board_id}")));
            assert_eq!(s.energy.get("pip"), Some(&0));
        }
    }

    fn owned(id: &str) -> Vec<CompanionProjection> {
        vec![CompanionProjection {
            id: id.into(),
            level: 1,
        }]
    }
    fn solve(
        def: &CampaignDefinition,
        start: &CampaignState,
        owned: &[CompanionProjection],
    ) -> CampaignState {
        let win_key = format!("win:{}", start.encounter.as_ref().unwrap().id);
        let mut queue = std::collections::VecDeque::from([start.clone()]);
        let mut seen = BTreeSet::new();
        while let Some(s) = queue.pop_front() {
            if s.encounter.is_none() {
                if s.grants.contains(&win_key) {
                    return s;
                }
                continue;
            }
            let key = serde_json::to_string(&(s.encounter.as_ref(), &s.energy)).unwrap();
            if !seen.insert(key) {
                continue;
            }
            for movement in ["focus", "signature", "tool", "guard", "rest"] {
                let c = CampaignCommand::EncounterTurn {
                    companion_id: s.team[0].clone(),
                    movement: movement.into(),
                };
                if let Ok(next) = apply(def, &s, &c, owned, false) {
                    if next.state.encounter.is_none() && next.state.location != s.location {
                        continue;
                    }
                    queue.push_back(next.state);
                }
            }
        }
        panic!("no supply-free solution")
    }
    #[test]
    fn all_starters_solve_both_trials_and_replay_without_rewards() {
        let def = definition();
        for starter in &def.starters {
            let friends = owned(starter);
            for board in def
                .encounters
                .iter()
                .filter(|e| !e.id.starts_with("practice_") && e.relay.is_none())
            {
                let mut s = apply(
                    &def,
                    &adopt(&def),
                    &CampaignCommand::ChooseStarter {
                        companion_id: starter.clone(),
                    },
                    &friends,
                    false,
                )
                .unwrap()
                .state;
                s.encounter = Some(EncounterState {
                    id: board.id.clone(),
                    progress: 0,
                    pressure: 0,
                    turns: 0,
                    tool_charges: 3,
                    partner_bonus: 0,
                    bonus_source: None,
                    replay: false,
                });
                let won = solve(&def, &s, &friends);
                assert!(won.flags.contains(&board.win_flags[0]));
                let grants = won.grants.clone();
                let items = won.items.clone();
                let mut replay = won.clone();
                replay.encounter = Some(EncounterState {
                    id: board.id.clone(),
                    progress: 0,
                    pressure: 0,
                    turns: 0,
                    tool_charges: 3,
                    partner_bonus: 0,
                    bonus_source: None,
                    replay: true,
                });
                replay.energy.insert(starter.clone(), 3);
                let again = solve(&def, &replay, &friends);
                assert_eq!(again.grants, grants);
                assert_eq!(again.items, items);
            }
        }
    }
    #[test]
    fn complete_campaign_for_every_starter_with_saved_replay() {
        for current_first in [false, true] {
            let mut def = definition();
            if current_first {
                def.locations
                    .iter_mut()
                    .find(|l| l.id == "orchard_hub")
                    .unwrap()
                    .routes
                    .reverse();
            }
            for starter in &def.starters {
                let mut friends = owned(starter);
                let mut state = apply(
                    &def,
                    &adopt(&def),
                    &CampaignCommand::ChooseStarter {
                        companion_id: starter.clone(),
                    },
                    &friends,
                    false,
                )
                .unwrap()
                .state;
                for _ in 0..1000 {
                    if state.flags.contains("finale_complete")
                        && friends.len() == 12
                        && friends.iter().all(|c| {
                            state
                                .flags
                                .contains(&format!("milestone:{}:wayfinder", c.id))
                        })
                    {
                        break;
                    }
                    if state.encounter.is_some() {
                        let board = &state.encounter.as_ref().unwrap().id;
                        if matches!(board.as_str(), "trial_harvest" | "trial_current")
                            && !state.flags.contains("harvest_complete")
                            && !state.flags.contains("current_complete")
                        {
                            assert_eq!(
                                board,
                                if current_first {
                                    "trial_current"
                                } else {
                                    "trial_harvest"
                                }
                            );
                        }
                        state = solve(&def, &state, &friends);
                        continue;
                    }
                    let public = snapshot(&def, &state, &friends).unwrap();
                    let next_action = public.legal_actions.iter().find(|a| {
                        !a.encounter
                            .as_ref()
                            .is_some_and(|id| state.grants.contains(&format!("win:{id}")))
                            && match &a.puzzle {
                                Some(p) => {
                                    let prefix = format!("puzzle:{}:", p.id);
                                    if state.flags.contains(&format!("{prefix}solved")) {
                                        false
                                    } else {
                                        let goal = def
                                            .actions
                                            .iter()
                                            .filter_map(|a| a.puzzle.as_ref())
                                            .find(|other| {
                                                other.id == p.id && other.operation == "check"
                                            })
                                            .unwrap()
                                            .goal
                                            .as_ref()
                                            .unwrap();
                                        if let Some(key) = p.operation.strip_prefix("set_") {
                                            p.value.as_ref() == goal.get(key)
                                                && !state.flags.contains(&format!(
                                                    "{prefix}{key}:{}",
                                                    p.value.as_ref().unwrap()
                                                ))
                                        } else if p.operation == "check" {
                                            goal.iter().all(|(k, v)| {
                                                state.flags.contains(&format!("{prefix}{k}:{v}"))
                                            })
                                        } else {
                                            false
                                        }
                                    }
                                }
                                None => true,
                            }
                    });
                    let command = if let Some(action) = next_action {
                        CampaignCommand::Action {
                            action_id: action.id.clone(),
                        }
                    } else {
                        // Search reachable locations for the nearest available authored action.
                        let mut queue = std::collections::VecDeque::from([(
                            state.location.clone(),
                            Vec::<String>::new(),
                        )]);
                        let mut seen = BTreeSet::new();
                        let mut path = None;
                        while let Some((location, route)) = queue.pop_front() {
                            if !seen.insert(location.clone()) {
                                continue;
                            }
                            let mut projected = state.clone();
                            projected.location = location.clone();
                            if !route.is_empty()
                                && snapshot(&def, &projected, &friends)
                                    .unwrap()
                                    .legal_actions
                                    .iter()
                                    .any(|a| {
                                        !a.encounter.as_ref().is_some_and(|id| {
                                            state.grants.contains(&format!("win:{id}"))
                                        }) && (a.puzzle.is_none()
                                            || a.puzzle.as_ref().is_some_and(|p| {
                                                !state
                                                    .flags
                                                    .contains(&format!("puzzle:{}:solved", p.id))
                                            }))
                                    })
                            {
                                path = Some(route);
                                break;
                            }
                            for target in snapshot(&def, &projected, &friends).unwrap().travel {
                                let mut next = route.clone();
                                next.push(target.clone());
                                queue.push_back((target, next));
                            }
                        }
                        CampaignCommand::Travel {
                            location_id: path.expect("chapter has a reachable next action")[0]
                                .clone(),
                        }
                    };
                    let result = apply(&def, &state, &command, &friends, true).unwrap();
                    for id in result.recruit {
                        if !friends.iter().any(|c| c.id == id) {
                            friends.push(CompanionProjection { id, level: 1 });
                        }
                    }
                    // Persisted JSON roundtrips retain deterministic state and command results.
                    let saved: CampaignState =
                        serde_json::from_str(&serde_json::to_string(&result.state).unwrap())
                            .unwrap();
                    assert_eq!(saved, result.state);
                    state = saved;
                }
                assert!(state.flags.contains("bridge_complete"));
                assert!(state.flags.contains("bloom_complete"));
                assert!(state.flags.contains("finale_complete"));
                assert_eq!(friends.len(), 12);
                assert!(friends.iter().all(|c| {
                    state
                        .flags
                        .contains(&format!("milestone:{}:wayfinder", c.id))
                }));
            }
        }
    }
    #[test]
    fn irrigation_wrong_route_reset_reload_and_both_creative_choices() {
        let def = definition();
        let friends = owned("pip");
        let action = |id: &str| CampaignCommand::Action {
            action_id: id.into(),
        };
        for design in ["garden_wild", "garden_ordered"] {
            let mut s = apply(
                &def,
                &adopt(&def),
                &CampaignCommand::ChooseStarter {
                    companion_id: "pip".into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            s.location = def
                .actions
                .iter()
                .find(|a| a.id == "water_check")
                .unwrap()
                .location
                .clone();
            s.flags.insert("bloom_clue_complete".into());
            for id in ["water_near_bypass", "water_far_collector", "water_check"] {
                s = apply(&def, &s, &action(id), &friends, false).unwrap().state;
            }
            assert!(s.flags.contains("puzzle:garden_water:blocked"));
            assert!(!s.flags.contains("puzzle:garden_water:solved"));
            s = apply(&def, &s, &action("water_reset"), &friends, false)
                .unwrap()
                .state;
            assert!(
                !s.flags
                    .iter()
                    .any(|f| f.starts_with("puzzle:garden_water:"))
            );
            for id in ["water_near_garden", "water_far_seedlings"] {
                s = apply(&def, &s, &action(id), &friends, false).unwrap().state;
            }
            s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
            s = apply(&def, &s, &action("water_check"), &friends, false)
                .unwrap()
                .state;
            assert!(s.flags.contains("puzzle:garden_water:solved"));
            s = apply(&def, &s, &action(design), &friends, false)
                .unwrap()
                .state;
            let other = if design == "garden_wild" {
                "garden_ordered"
            } else {
                "garden_wild"
            };
            assert!(apply(&def, &s, &action(other), &friends, false).is_err());
            assert!(s.flags.contains(design));
        }
    }
    #[test]
    fn rally_bonus_is_consumed_only_by_partner_and_victory_precedes_hazard() {
        let mut def = definition();
        def.encounters[0].target = 5;
        def.encounters[0].hazards = vec![99];
        let friends = vec![
            CompanionProjection {
                id: "pip".into(),
                level: 2,
            },
            CompanionProjection {
                id: "nib".into(),
                level: 1,
            },
        ];
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s.team.push("nib".into());
        s.energy.insert("nib".into(), 3);
        s.encounter = Some(EncounterState {
            id: def.encounters[0].id.clone(),
            progress: 0,
            pressure: 0,
            turns: 0,
            tool_charges: 3,
            partner_bonus: 2,
            bonus_source: Some("pip".into()),
            replay: false,
        });
        let result = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "nib".into(),
                movement: "signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(result.encounter.is_none());
        assert!(result.flags.contains("bridge_seal"));
        assert_eq!(result.energy["nib"], 2);
    }
    #[test]
    fn all_twelve_have_four_once_only_contextual_milestones() {
        let def = definition();
        let friends: Vec<_> = def
            .companions
            .iter()
            .map(|c| CompanionProjection {
                id: c.id.clone(),
                level: 1,
            })
            .collect();
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        for unlock in [
            "welcome_complete",
            "echo_complete",
            "current_complete",
            "beacon_complete",
        ] {
            s.flags.insert(unlock.into());
        }
        for friend in &friends {
            for milestone in [
                "first_help",
                "field_partnership",
                "personal_project",
                "wayfinder",
            ] {
                let action = def
                    .actions
                    .iter()
                    .find(|a| {
                        a.recipient.as_ref() == Some(&friend.id)
                            && a.milestone.as_deref() == Some(milestone)
                    })
                    .expect("explicit contextual milestone for every friend");
                s.location = action.location.clone();
                let command = CampaignCommand::Action {
                    action_id: action.id.clone(),
                };
                assert!(
                    snapshot(&def, &s, &friends)
                        .unwrap()
                        .legal_actions
                        .iter()
                        .any(|a| a.id == action.id)
                );
                assert!(matches!(
                    apply(&def, &s, &command, &friends, false),
                    Err(CampaignError::LearningRequired)
                ));
                s = apply(&def, &s, &command, &friends, true).unwrap().state;
                assert!(
                    s.flags
                        .contains(&format!("milestone:{}:{milestone}", friend.id))
                );
                assert!(apply(&def, &s, &command, &friends, true).is_err());
            }
        }
    }
    #[test]
    fn specialization_declined_offer_and_late_camp_survive_reload() {
        let def = definition();
        let friends = vec![CompanionProjection {
            id: "pip".into(),
            level: 3,
        }];
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s = apply(
            &def,
            &s,
            &CampaignCommand::Care {
                companion_id: "pip".into(),
                kind: "specialize_free_signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s.energy.insert("pip".into(), 0);
        s.encounter = Some(EncounterState {
            id: "trial_bridge".into(),
            progress: 0,
            pressure: 0,
            turns: 0,
            tool_charges: 3,
            partner_bonus: 0,
            bonus_source: None,
            replay: false,
        });
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert_eq!(s.energy["pip"], 0);
        assert!(s.flags.contains("free_signature_used:pip"));
        assert!(
            apply(
                &def,
                &s,
                &CampaignCommand::EncounterTurn {
                    companion_id: "pip".into(),
                    movement: "signature".into()
                },
                &friends,
                false
            )
            .is_err()
        );
        s.flags.insert("last_camp:harbour_lookout".into());
        recover(&def, &mut s, &friends);
        assert_eq!(s.location, "harbour_lookout");
        assert_eq!(s.energy["pip"], 4);
        let invitation = def
            .actions
            .iter()
            .find(|a| a.recruit.as_deref() == Some("tansy"))
            .unwrap();
        s.location = invitation.location.clone();
        s.flags.extend(invitation.requires_flags.clone());
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: format!("decline:{}", invitation.id),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(
            s.flags
                .contains(&format!("recruit_declined:{}", invitation.id))
        );
        assert!(
            snapshot(&def, &s, &friends)
                .unwrap()
                .legal_actions
                .iter()
                .any(|a| a.id == invitation.id)
        );
    }
    #[test]
    fn bounded_guidance_handles_late_depleted_pair_and_retreat_is_safe() {
        let def = definition();
        let friends = vec![
            CompanionProjection {
                id: "pip".into(),
                level: 3,
            },
            CompanionProjection {
                id: "nib".into(),
                level: 3,
            },
        ];
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s.team.push("nib".into());
        s.energy.insert("pip".into(), 0);
        s.energy.insert("nib".into(), 0);
        s.flags.insert("last_camp:harbour_lookout".into());
        s.flags.insert("technique:pip:free_signature".into());
        s.flags.insert("free_signature_used:pip".into());
        s.encounter = Some(EncounterState {
            id: "trial_accord".into(),
            progress: 0,
            pressure: 0,
            turns: 0,
            tool_charges: 3,
            partner_bonus: 0,
            bonus_source: None,
            replay: false,
        });
        let command = guided_command(&def, &s, &friends).unwrap();
        let next = apply(&def, &s, &command, &friends, false).unwrap();
        assert!(next.state.encounter.is_some() || next.state.location == "harbour_lookout");
        let before = s.items.clone();
        let recovered = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "retreat".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(recovered.encounter.is_none());
        assert_eq!(recovered.location, "harbour_lookout");
        assert_eq!(recovered.items, before);
        assert_eq!(recovered.energy["nib"], 4);
    }
    #[test]
    fn prior_first_help_does_not_block_required_project_and_public_puzzles_hide_goals() {
        let def = definition();
        let friends = owned("pip");
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s.flags.insert("welcome_complete".into());
        s.flags.insert("bridge_gather_done".into());
        let training = def
            .actions
            .iter()
            .find(|a| a.id == "train_pip_first_help")
            .unwrap();
        s.location = training.location.clone();
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: training.id.clone(),
            },
            &friends,
            true,
        )
        .unwrap()
        .state;
        let project = def.actions.iter().find(|a| a.id == "bridge_plan").unwrap();
        s.location = project.location.clone();
        assert!(
            snapshot(&def, &s, &friends)
                .unwrap()
                .legal_actions
                .iter()
                .any(|a| a.id == project.id)
        );
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: project.id.clone(),
            },
            &friends,
            true,
        )
        .unwrap()
        .state;
        assert!(s.flags.contains("bridge_plan_ready"));
        s.flags.insert("bloom_clue_complete".into());
        s.location = def
            .actions
            .iter()
            .find(|a| a.id == "water_check")
            .unwrap()
            .location
            .clone();
        let public = snapshot(&def, &s, &friends).unwrap();
        assert!(
            public
                .legal_actions
                .iter()
                .filter_map(|a| a.puzzle.as_ref())
                .all(|p| p.goal.is_none())
        );
        assert!(
            def.actions
                .iter()
                .find(|a| a.id == "water_check")
                .unwrap()
                .puzzle
                .as_ref()
                .unwrap()
                .goal
                .is_some()
        );
    }
    #[test]
    fn gleam_reveal_persists_one_turn_and_clears_on_rest_or_recovery() {
        let def = definition();
        let friends = owned("pip");
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s.encounter = Some(EncounterState {
            id: "trial_bridge".into(),
            progress: 0,
            pressure: 0,
            turns: 0,
            tool_charges: 3,
            partner_bonus: 0,
            bonus_source: None,
            replay: false,
        });
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(s.flags.contains("encounter_reveal:trial_bridge:1"));
        s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(
            snapshot(&def, &s, &friends)
                .unwrap()
                .state
                .flags
                .contains("encounter_reveal:trial_bridge:1")
        );
        let revealed = s.clone();
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "rest".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(!s.flags.iter().any(|f| f.starts_with("encounter_reveal:")));
        s = apply(
            &def,
            &revealed,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "retreat".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(!s.flags.iter().any(|f| f.starts_with("encounter_reveal:")));
    }
    #[test]
    fn discovery_mismatch_keeps_preference_and_invitation_available_after_reload() {
        let def = definition();
        let friends = owned("nib");
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "nib".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        let observe = def.actions.iter().find(|a| a.id == "observe_pip").unwrap();
        s.location = observe.location.clone();
        s.flags.extend(observe.requires_flags.clone());
        let supplies = s.items.clone();
        for id in ["observe_pip", "approach_pip_play"] {
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: id.into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
        }
        assert!(s.flags.contains("discovery:pip:clue_play"));
        assert_eq!(s.items, supplies);
        s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(
            snapshot(&def, &s, &friends)
                .unwrap()
                .legal_actions
                .iter()
                .any(|a| a.id == "approach_pip_wait")
        );
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: "approach_pip_wait".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(s.flags.contains("discovery:pip:ready"));
        s.location = def
            .actions
            .iter()
            .find(|a| a.id == "help_pip")
            .unwrap()
            .location
            .clone();
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: "help_pip".into(),
            },
            &friends,
            true,
        )
        .unwrap()
        .state;
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: "decline:invite_pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(
            snapshot(&def, &s, &friends)
                .unwrap()
                .legal_actions
                .iter()
                .any(|a| a.id == "invite_pip")
        );
        let mut now_owned = friends.clone();
        now_owned.push(CompanionProjection {
            id: "pip".into(),
            level: 1,
        });
        assert!(
            !snapshot(&def, &s, &now_owned)
                .unwrap()
                .legal_actions
                .iter()
                .any(|a| a.discovery_for.as_deref() == Some("pip"))
        );
        assert!(
            apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: "invite_pip".into()
                },
                &now_owned,
                false
            )
            .is_err()
        );
    }
    #[test]
    fn relay_has_three_distinct_rounds_carries_resources_and_requires_two_signatures() {
        let mut def = definition();
        let board = def
            .encounters
            .iter_mut()
            .find(|e| e.id == "rowan_final_board")
            .unwrap();
        board.relay = Some(RelayDefinition {
            round_targets: vec![3, 6, 9],
            required_affinities: 2,
        });
        board.target = 9;
        board.phases = vec![
            Phase {
                threshold: 0,
                affinity: "growth".into(),
            },
            Phase {
                threshold: 3,
                affinity: "rhythm".into(),
            },
            Phase {
                threshold: 6,
                affinity: "gleam".into(),
            },
        ];
        validate(&def).unwrap();
        let friends = vec![
            CompanionProjection {
                id: "pip".into(),
                level: 3,
            },
            CompanionProjection {
                id: "nib".into(),
                level: 3,
            },
            CompanionProjection {
                id: "luna".into(),
                level: 3,
            },
        ];
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        let action = def
            .actions
            .iter()
            .find(|a| a.encounter.as_deref() == Some("rowan_final_board"))
            .unwrap();
        s.location = action.location.clone();
        s.flags.extend(action.requires_flags.clone());
        s.team = vec!["pip".into(), "luna".into()];
        assert!(
            apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: action.id.clone()
                },
                &friends,
                false
            )
            .is_err()
        );
        s.team = vec!["pip".into(), "nib".into()];
        s.energy.insert("nib".into(), 4);
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: action.id.clone(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        for (actor, movement) in [("pip", "signature"), ("pip", "focus")] {
            s = apply(
                &def,
                &s,
                &CampaignCommand::EncounterTurn {
                    companion_id: actor.into(),
                    movement: movement.into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
        }
        assert_eq!(s.encounter.as_ref().unwrap().progress, 3);
        assert_eq!(s.encounter.as_ref().unwrap().pressure, 1);
        assert_eq!(s.encounter.as_ref().unwrap().turns, 2);
        assert_eq!(s.energy["pip"], 3);
        assert_eq!(s.encounter.as_ref().unwrap().tool_charges, 3);
        assert!(s.flags.contains("relay:rowan_final_board:round:1"));
        // A large following move cannot skip a round; actor affinities come from signatures.
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "tool".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert_eq!(s.encounter.as_ref().unwrap().progress, 6);
        assert_eq!(s.encounter.as_ref().unwrap().tool_charges, 2);
        assert_eq!(s.encounter.as_ref().unwrap().pressure, 3);
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert_eq!(s.encounter.as_ref().unwrap().progress, 8);
        assert!(!s.flags.contains("rowan_final_won"));
        assert!(!s.flags.contains("relay:rowan_final_board:round:3"));
        s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "nib".into(),
                movement: "signature".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(s.encounter.is_none());
        assert!(s.flags.contains("rowan_final_won"));
        assert!(s.flags.contains("relay:rowan_final_board:round:3"));
        let grants = s.grants.clone();
        let supplies = s.items.clone();
        s = apply(
            &def,
            &s,
            &CampaignCommand::Action {
                action_id: action.id.clone(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert!(
            !s.flags
                .iter()
                .any(|f| f.starts_with("relay:rowan_final_board:"))
        );
        assert_eq!(s.encounter.as_ref().unwrap().tool_charges, 3);
        s = apply(
            &def,
            &s,
            &CampaignCommand::EncounterTurn {
                companion_id: "pip".into(),
                movement: "retreat".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        assert_eq!(s.grants, grants);
        assert_eq!(s.items, supplies);
    }
    #[test]
    fn remembered_introductions_are_exclusive_and_only_selected_intro_is_recorded() {
        let def = definition();
        let friends = owned("pip");
        for remembered in [false, true] {
            let mut s = apply(
                &def,
                &adopt(&def),
                &CampaignCommand::ChooseStarter {
                    companion_id: "pip".into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            s.location = "harbour_lookout".into();
            s.flags.insert("cove_lookout_friendship".into());
            if remembered {
                s.flags.insert("lens_cradle_seen".into());
            }
            let selected = if remembered {
                "observe_sail_remembered"
            } else {
                "observe_sail"
            };
            let other = if remembered {
                "observe_sail"
            } else {
                "observe_sail_remembered"
            };
            assert!(
                snapshot(&def, &s, &friends)
                    .unwrap()
                    .legal_actions
                    .iter()
                    .any(|a| a.id == selected)
            );
            assert!(
                apply(
                    &def,
                    &s,
                    &CampaignCommand::Action {
                        action_id: other.into()
                    },
                    &friends,
                    false
                )
                .is_err()
            );
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: selected.into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            assert!(s.flags.contains("discovery:sail:observed"));
            assert!(
                !snapshot(&def, &s, &friends)
                    .unwrap()
                    .legal_actions
                    .iter()
                    .any(|a| a.id.starts_with("observe_sail"))
            );
        }
        for response in [None, Some("kind"), Some("direct")] {
            let mut s = apply(
                &def,
                &adopt(&def),
                &CampaignCommand::ChooseStarter {
                    companion_id: "pip".into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            s.location = "courier_pond".into();
            s.flags.insert("current_complete".into());
            if let Some(response) = response {
                s.flags.insert(format!("rowan_response_{response}"));
            }
            let selected = match response {
                None => "rowan_shared",
                Some("kind") => "rowan_shared_remembered",
                _ => "rowan_shared_direct",
            };
            let intros = [
                "rowan_shared",
                "rowan_shared_remembered",
                "rowan_shared_direct",
            ];
            assert_eq!(
                snapshot(&def, &s, &friends)
                    .unwrap()
                    .legal_actions
                    .iter()
                    .filter(|a| intros.contains(&a.id.as_str()))
                    .count(),
                1
            );
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: selected.into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            s = solve(&def, &s, &friends);
            assert!(s.flags.contains("rowan_shared_done"));
            assert!(s.grants.contains(&format!("action:{selected}")));
            for other in intros.into_iter().filter(|id| *id != selected) {
                assert!(!s.grants.contains(&format!("action:{other}")));
            }
        }
    }
    #[test]
    fn fossil_original_and_exchanged_copies_open_distinct_once_only_discoveries() {
        let def = definition();
        let friends = owned("pip");
        for first in ["leaf", "shell"] {
            let other = if first == "leaf" { "shell" } else { "leaf" };
            let mut s = apply(
                &def,
                &adopt(&def),
                &CampaignCommand::ChooseStarter {
                    companion_id: "pip".into(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            s.location = "old_mill".into();
            s.flags.insert("harvest_celebrate_done".into());
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: format!("fossil_restore_{first}"),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            assert_eq!(s.items["duplicate_rubbing"], 2);
            let original = format!("fossil_{first}_explore_original");
            let copy = format!("fossil_{other}_explore_copy");
            s.location = def
                .actions
                .iter()
                .find(|a| a.id == copy)
                .unwrap()
                .location
                .clone();
            assert!(
                apply(
                    &def,
                    &s,
                    &CampaignCommand::Action {
                        action_id: copy.clone()
                    },
                    &friends,
                    false
                )
                .is_err()
            );
            s.location = def
                .actions
                .iter()
                .find(|a| a.id == original)
                .unwrap()
                .location
                .clone();
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: original.clone(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            assert!(
                apply(
                    &def,
                    &s,
                    &CampaignCommand::Action {
                        action_id: original
                    },
                    &friends,
                    false
                )
                .is_err()
            );
            s.location = "old_mill".into();
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: format!("fossil_trade_{first}"),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            assert_eq!(s.items["duplicate_rubbing"], 1);
            assert_eq!(s.items[&format!("{first}_fossil_keepsake")], 1);
            s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
            s.location = def
                .actions
                .iter()
                .find(|a| a.id == copy)
                .unwrap()
                .location
                .clone();
            s = apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: copy.clone(),
                },
                &friends,
                false,
            )
            .unwrap()
            .state;
            assert!(s.flags.contains("fossil_leaf_explored"));
            assert!(s.flags.contains("fossil_shell_explored"));
            let quantities = s.items.clone();
            assert!(
                apply(
                    &def,
                    &s,
                    &CampaignCommand::Action { action_id: copy },
                    &friends,
                    false
                )
                .is_err()
            );
            assert_eq!(s.items, quantities);
        }
    }
    #[test]
    fn late_two_friend_guidance_ignores_large_history_preserves_authority_and_finishes() {
        let def = definition();
        let friends = vec![
            CompanionProjection {
                id: "pip".into(),
                level: 5,
            },
            CompanionProjection {
                id: "tansy".into(),
                level: 5,
            },
        ];
        let mut compact = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        compact.team = vec!["pip".into(), "tansy".into()];
        compact.location = "shell_beach".into();
        compact.energy.insert("pip".into(), 5);
        compact.energy.insert("tansy".into(), 5);
        compact.flags.insert("technique:pip:free_signature".into());
        compact.flags.insert("free_signature_used:pip".into());
        compact.flags.insert("technique:tansy:rally".into());
        compact.encounter = Some(EncounterState {
            id: "postgame_tide".into(),
            progress: 0,
            pressure: 0,
            turns: 0,
            tool_charges: 3,
            partner_bonus: 0,
            bonus_source: None,
            replay: false,
        });
        let mut full = compact.clone();
        for index in 0..466 {
            full.flags.insert(format!("history:quest:{index:03}"));
        }
        for index in 0..411 {
            full.grants.insert(format!("campaign:prior:{index:03}"));
        }
        for index in 0..40 {
            full.items.insert(format!("keepsake:{index:03}"), 1);
        }
        let history = full.flags.clone();
        let grants = full.grants.clone();
        let items = full.items.clone();
        let started = std::time::Instant::now();
        let command = guided_command(&def, &full, &friends).unwrap();
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "late history must not approach the unchanged 10s HTTP timeout"
        );
        assert_eq!(
            serde_json::to_value(&command).unwrap(),
            serde_json::to_value(guided_command(&def, &compact, &friends).unwrap()).unwrap()
        );
        assert!(
            !matches!(&command,CampaignCommand::EncounterTurn{movement,..}if movement=="retreat"),
            "fresh late-game pair has a supply-free continuation"
        );
        let before = full.clone();
        let result = apply(&def, &full, &command, &friends, false).unwrap();
        assert_eq!(full, before);
        full = result.state;
        assert!(history.is_subset(&full.flags));
        assert!(grants.is_subset(&full.grants));
        assert_eq!(full.items, items);
        for _ in 0..15 {
            if full.encounter.is_none() {
                break;
            }
            let command = guided_command(&def, &full, &friends).unwrap();
            assert!(
                !matches!(&command,CampaignCommand::EncounterTurn{movement,..}if movement=="retreat")
            );
            full = apply(&def, &full, &command, &friends, false).unwrap().state;
        }
        assert!(full.flags.contains("postgame_tide_won"));
        assert!(grants.is_subset(&full.grants));
        assert!(history.is_subset(&full.flags));
        // A depleted pair uses the same bounded planner and remains authoritatively recoverable.
        compact.energy.insert("pip".into(), 0);
        compact.energy.insert("tansy".into(), 0);
        let mut depleted = compact.clone();
        depleted.flags.extend(history);
        depleted.grants.extend(grants);
        depleted.items.extend(items);
        let started = std::time::Instant::now();
        let command = guided_command(&def, &depleted, &friends).unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
        assert_eq!(
            serde_json::to_value(&command).unwrap(),
            serde_json::to_value(guided_command(&def, &compact, &friends).unwrap()).unwrap()
        );
        assert!(apply(&def, &depleted, &command, &friends, false).is_ok());
    }
    #[test]
    fn starter_pairs_guidance_wins_every_campaign_board_in_both_orders() {
        let def = definition();
        for pair in [["nib", "pip"], ["nib", "tansy"], ["pip", "tansy"]] {
            let friends = vec![
                CompanionProjection {
                    id: pair[0].into(),
                    level: 5,
                },
                CompanionProjection {
                    id: pair[1].into(),
                    level: 5,
                },
            ];
            for reverse in [false, true] {
                for energy in [0, 5] {
                    for board in def
                        .encounters
                        .iter()
                        .filter(|e| !e.id.starts_with("practice_"))
                    {
                        let mut s = adopt(&def);
                        s.team = if reverse {
                            vec![pair[1].into(), pair[0].into()]
                        } else {
                            vec![pair[0].into(), pair[1].into()]
                        };
                        for id in &s.team {
                            s.energy.insert(id.clone(), energy);
                        }
                        s.encounter = Some(EncounterState {
                            id: board.id.clone(),
                            progress: 0,
                            pressure: 0,
                            turns: 0,
                            tool_charges: 3,
                            partner_bonus: 0,
                            bonus_source: None,
                            replay: false,
                        });
                        let started = std::time::Instant::now();
                        for _ in 0..board.turn_limit {
                            if s.encounter.is_none() {
                                break;
                            }
                            let command = guided_command(&def, &s, &friends).unwrap();
                            assert!(
                                !matches!(&command, CampaignCommand::EncounterTurn {movement, ..} if movement == "retreat"),
                                "{} energy {energy} reverse {reverse}",
                                board.id
                            );
                            s = apply(&def, &s, &command, &friends, false).unwrap().state;
                        }
                        assert!(
                            s.grants.contains(&format!("win:{}", board.id)),
                            "{} energy {energy}",
                            board.id
                        );
                        assert!(started.elapsed() < std::time::Duration::from_secs(5));
                    }
                }
            }
        }
    }
    #[test]
    fn guidance_retreats_when_no_legal_supply_free_move_can_survive() {
        let def = definition();
        let friends = vec![
            CompanionProjection {
                id: "nib".into(),
                level: 5,
            },
            CompanionProjection {
                id: "pip".into(),
                level: 5,
            },
        ];
        let mut s = adopt(&def);
        s.team = vec!["nib".into(), "pip".into()];
        s.energy.insert("nib".into(), 0);
        s.energy.insert("pip".into(), 0);
        s.encounter = Some(EncounterState {
            id: "postgame_beacon".into(),
            progress: 0,
            pressure: 7,
            turns: 0,
            tool_charges: 0,
            partner_bonus: 0,
            bonus_source: None,
            replay: false,
        });
        s.flags.insert("last_camp:harbour_lookout".into());
        s.items.insert("snack".into(), 2);
        let command = guided_command(&def, &s, &friends).unwrap();
        assert!(
            matches!(&command, CampaignCommand::EncounterTurn {movement, ..} if movement == "retreat")
        );
        let recovered = apply(&def, &s, &command, &friends, false).unwrap().state;
        assert!(recovered.encounter.is_none());
        assert_eq!(recovered.location, "harbour_lookout");
        assert_eq!(recovered.items, s.items);
        assert_eq!(recovered.grants, s.grants);
    }
    #[test]
    fn invalid_learning_and_insufficient_items_leave_source_unchanged() {
        let def = definition();
        let friends = owned("pip");
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        s.flags.insert("met_starters".into());
        let before = s.clone();
        assert!(matches!(
            apply(
                &def,
                &s,
                &CampaignCommand::Action {
                    action_id: "welcome_supplies".into()
                },
                &friends,
                false
            ),
            Err(CampaignError::LearningRequired)
        ));
        assert_eq!(s, before);
        s.items.clear();
        assert!(
            apply(
                &def,
                &s,
                &CampaignCommand::Craft {
                    recipe: "snack".into()
                },
                &friends,
                false
            )
            .is_err()
        );
        assert!(s.items.is_empty());
    }
    #[test]
    fn adoption_versions_and_camp_cannot_farm() {
        let def = definition();
        let friends = owned("pip");
        let mut s = apply(
            &def,
            &adopt(&def),
            &CampaignCommand::ChooseStarter {
                companion_id: "pip".into(),
            },
            &friends,
            false,
        )
        .unwrap()
        .state;
        let start = s.items.clone();
        for _ in 0..4 {
            s = apply(&def, &s, &CampaignCommand::Camp, &friends, false)
                .unwrap()
                .state;
        }
        assert_eq!(s.items, start);
        s.content_version = "unknown".into();
        assert!(matches!(
            snapshot(&def, &s, &friends),
            Err(CampaignError::VersionMismatch)
        ));
    }
}
