//! Typed public projections. Internal receipts, answer keys and saved state stay private.
use super::{Aggregate, Error, digest_id};
use serde_json::Value;
use std::collections::BTreeSet;
use wonderwilds_contracts as wire;
use wonderwilds_domain::engine::{self, Content};
use wonderwilds_learning::assessment::wire::authored_encounter;
use wonderwilds_learning::mastery::{self, Observation, Status};

impl Aggregate {
    pub fn snapshot(&self, content: &Content) -> Result<Value, Error> {
        let snapshot = wire::GameSnapshot {
            schema_version: wire::GameSnapshotSchemaVersion::V10,
            revision: self.revision,
            content_version: content.version().into(),
            rules_version: engine::RULES_VERSION.into(),
            story_flags: self.domain.flags().iter().cloned().collect(),
            owned_creatures: self.domain.creatures().iter().cloned().collect(),
            inventory: self
                .domain
                .inventory()
                .iter()
                .map(|(id, quantity)| wire::InventoryEntry {
                    item_id: id.clone(),
                    quantity: u64::from(*quantity),
                })
                .collect(),
            pending_encounters: self
                .encounters
                .values()
                .filter(|e| !e.completed && !e.abandoned)
                .map(|e| self.encounter_projection(&e.id))
                .collect::<Result<_, _>>()?,
            learner_summaries: self.learner_summaries()?,
            selected_companions: Some(self.domain.team().to_vec()),
            friendship: Some(
                self.domain
                    .creatures()
                    .iter()
                    .map(|id| wire::FriendshipEntry {
                        creature_id: id.clone(),
                        points: u64::from(self.domain.friendship(id)),
                    })
                    .collect(),
            ),
            transformed_creatures: Some(self.domain.transformed().iter().cloned().collect()),
            available_helping_moments: Some(
                self.domain
                    .available_helping_moments(content)
                    .into_iter()
                    .map(|(creature_id, completed_quest_id)| wire::HelpingMoment {
                        creature_id,
                        completed_quest_id,
                    })
                    .collect(),
            ),
            character_progression: Some(self.character_progression(content)),
            explorer_progression: Some(wire::ExplorerProgression {
                xp: u64::from(self.domain.xp()),
                level: self.domain.level(content) as u64,
            }),
            team_capacity: Some(self.domain.team_capacity() as u64),
        };
        let value = serde_json::to_value(snapshot).map_err(|_| Error::Unavailable)?;
        wire::validate("GameSnapshot", &value).map_err(|_| Error::Unavailable)?;
        Ok(value)
    }

    fn learner_summaries(&self) -> Result<Vec<wire::LearnerSummary>, Error> {
        let mut observations = Vec::new();
        let mut skills = BTreeSet::new();
        for evidence in &self.evidence {
            let saved = self
                .encounters
                .get(&evidence.encounter_id)
                .ok_or(Error::Unavailable)?;
            if saved.skill != evidence.skill_id {
                return Err(Error::Unavailable);
            }
            let (_, canonical) = authored_encounter(&saved.skill, saved.seed, &saved.id)
                .map_err(|_| Error::Unavailable)?;
            let parameters =
                serde_json::to_string(&canonical.parameters).map_err(|_| Error::Unavailable)?;
            observations.push(Observation {
                evidence: evidence.clone(),
                parameter_fingerprint: digest_id(&parameters),
                // Reopening a draft authored item never invents reviewed novel transfer.
                novel_transfer: false,
            });
            skills.insert(evidence.skill_id.clone());
        }
        skills
            .into_iter()
            .map(|skill_id| {
                let summary = mastery::recompute(&skill_id, &observations, 2)
                    .map_err(|_| Error::Unavailable)?;
                let status = match summary.status {
                    Status::Unseen => wire::LearnerSummaryStatus::Unseen,
                    Status::Introduced => wire::LearnerSummaryStatus::Introduced,
                    Status::Practising => wire::LearnerSummaryStatus::Practising,
                    Status::Secure => wire::LearnerSummaryStatus::Secure,
                };
                Ok(wire::LearnerSummary {
                    skill_id,
                    status,
                    explanation: summary.explanation.into(),
                })
            })
            .collect()
    }

    fn character_progression(&self, content: &Content) -> Vec<wire::CharacterProgression> {
        let available = self.domain.available_training(content);
        self.domain
            .creatures()
            .iter()
            .map(|id| {
                let level = self.domain.character_level(content, id);
                wire::CharacterProgression {
                    creature_id: id.clone(),
                    xp: u64::from(self.domain.character_xp(id)),
                    level: level as u64,
                    next_level_xp: content
                        .character_level_thresholds()
                        .get(level)
                        .map(|xp| u64::from(*xp)),
                    reaction_id: content
                        .training_definitions()
                        .filter(|t| t.creature_id == *id && self.domain.training_completed(&t.id))
                        .max_by_key(|t| t.required_level)
                        .map(|t| t.reaction_id.clone()),
                    completed_training_ids: content
                        .training_definitions()
                        .filter(|t| t.creature_id == *id && self.domain.training_completed(&t.id))
                        .map(|t| t.id.clone())
                        .collect(),
                    available_training: available
                        .iter()
                        .filter(|t| t.creature_id == *id)
                        .map(|t| wire::TrainingOffer {
                            training_id: t.id.clone(),
                            required_level: t.required_level as u64,
                            xp: u64::from(t.xp),
                            reaction_id: t.reaction_id.clone(),
                        })
                        .collect(),
                }
            })
            .collect()
    }

    fn encounter_projection(&self, id: &str) -> Result<wire::EncounterInstance, Error> {
        let saved = self.encounters.get(id).ok_or(Error::NotFound)?;
        if saved.completed || saved.abandoned {
            return Err(Error::NotFound);
        }
        let (_, mut encounter) = authored_encounter(&saved.skill, saved.seed, &saved.id)
            .map_err(|_| Error::Unavailable)?;
        encounter.support = saved.support.clone();
        encounter.training = saved.training.clone();
        Ok(encounter)
    }

    pub fn public_encounter(&self, id: &str) -> Result<Value, Error> {
        serde_json::to_value(self.encounter_projection(id)?).map_err(|_| Error::Unavailable)
    }
}
