//! Deterministic assessment. Tasks and answers are server-internal, never browser authority.
//! This module deliberately has no clock, RNG, persistence or provider dependency.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Task {
    NumberSequence {
        start: u8,
        step: u8,
        shown: u8,
    },
    MissingQuantity {
        total: u8,
        given: u8,
    },
    EqualSharing {
        total: u8,
        recipients: u8,
    },
    Pattern {
        unit: Vec<String>,
        prefix_len: u8,
    },
    Sort {
        attributes: Vec<String>,
        target: String,
    },
    Position {
        relation: String,
        choices: Vec<String>,
    },
    Measure {
        lengths: Vec<u8>,
    },
    Word {
        tiles: Vec<String>,
        accepted: Vec<String>,
        independently_read: bool,
    },
    StorySequence {
        event_ids: Vec<String>,
    },
    MessageChoice {
        choices: Vec<String>,
        accepted: Vec<String>,
        independently_read: bool,
    },
    AdultReviewed {
        prompt_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Number(u8),
    Groups(Vec<u8>),
    Text(String),
    Indices(Vec<usize>),
    Tiles(Vec<usize>),
    EventIds(Vec<String>),
    AdultResponse,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Support {
    pub hint_count: u8,
    pub worked_example: bool,
    pub read_aloud: bool,
    pub motor_access: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Independent,
    Supported,
    Unassessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    /// None means this activity requires adult review; it is never automatic success.
    pub correct: Option<bool>,
    pub classification: Classification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidTask,
    InvalidAnswer,
    UnsupportedRepresentation,
}

fn label(value: &str) -> bool {
    !value.is_empty() && value.len() <= 80 && !value.chars().any(char::is_control)
}

fn labels(values: &[String], max: usize) -> bool {
    !values.is_empty() && values.len() <= max && values.iter().all(|s| label(s))
}

fn unique<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| !values[..i].contains(v))
}

impl Task {
    /// Bind an authored task to the draft curriculum slice. A valid mathematical task
    /// outside a node's bounds is still unsuitable for that node.
    pub fn validate_skill(&self, skill_id: &str) -> Result<(), Error> {
        self.validate()?;
        let allowed = match (skill_id, self) {
            ("M05", Self::NumberSequence { start, step, shown }) => {
                *step == 1 && (3..=10).contains(&(u16::from(*start) + u16::from(*shown)))
            }
            ("M06", Self::MissingQuantity { total, .. }) => (2..=10).contains(total),
            ("M08", Self::EqualSharing { total, .. }) => (2..=12).contains(total),
            ("M09", Self::Pattern { unit, prefix_len }) => {
                (2..=4).contains(&(usize::from(*prefix_len) / unit.len()))
            }
            ("M14", Self::Sort { attributes, .. }) => {
                let mut categories = attributes.clone();
                categories.sort();
                categories.dedup();
                (2..=3).contains(&categories.len())
            }
            ("M11", Self::Position { .. }) => true,
            ("M12", Self::Measure { lengths }) => (2..=3).contains(&lengths.len()),
            ("E06" | "E07", Self::Word { accepted, .. }) => accepted
                .iter()
                .all(|word| (2..=3).contains(&word.chars().count())),
            ("E05", Self::Word { accepted, .. }) => {
                accepted.iter().all(|word| word.chars().count() == 1)
            }
            ("E11", Self::StorySequence { event_ids }) => (2..=4).contains(&event_ids.len()),
            ("E10" | "E12", Self::MessageChoice { choices, .. }) => choices
                .iter()
                .all(|text| (3..=8).contains(&text.split_whitespace().count())),
            (
                "E01",
                Self::MessageChoice {
                    independently_read, ..
                },
            ) => !independently_read,
            ("E13" | "E14" | "E15", Self::AdultReviewed { .. }) => true,
            _ => false,
        };
        allowed.then_some(()).ok_or(Error::InvalidTask)
    }

    pub fn validate(&self) -> Result<(), Error> {
        let valid = match self {
            Self::NumberSequence { start, step, shown } => {
                *step > 0
                    && (2..=8).contains(shown)
                    && u16::from(*start) + u16::from(*step) * u16::from(*shown) <= 20
            }
            Self::MissingQuantity { total, given } => *total <= 20 && given <= total,
            Self::EqualSharing { total, recipients } => {
                *total <= 20 && (2..=5).contains(recipients) && total % recipients == 0
            }
            Self::Pattern { unit, prefix_len } => {
                labels(unit, 4)
                    && unit.len() >= 2
                    && unique(unit)
                    && *prefix_len >= unit.len() as u8
                    && *prefix_len <= 20
            }
            Self::Sort { attributes, target } => {
                labels(attributes, 20) && label(target) && attributes.contains(target)
            }
            Self::Position { relation, choices } => {
                ["above", "below", "beside", "inside"].contains(&relation.as_str())
                    && labels(choices, 4)
                    && unique(choices)
                    && choices.contains(relation)
            }
            Self::Measure { lengths } => {
                (2..=5).contains(&lengths.len())
                    && lengths.iter().all(|n| (1..=20).contains(n))
                    && lengths
                        .iter()
                        .filter(|n| Some(*n) == lengths.iter().max())
                        .count()
                        == 1
            }
            Self::Word {
                tiles, accepted, ..
            } => {
                labels(tiles, 12)
                    && tiles.iter().all(|s| {
                        s.chars().count() == 1 && s.chars().all(|c| c.is_ascii_lowercase())
                    })
                    && labels(accepted, 4)
                    && unique(accepted)
                    && accepted.iter().all(|s| word_can_be_built(tiles, s))
            }
            Self::StorySequence { event_ids } => {
                labels(event_ids, 8) && event_ids.len() >= 2 && unique(event_ids)
            }
            Self::MessageChoice {
                choices, accepted, ..
            } => {
                labels(choices, 6)
                    && unique(choices)
                    && labels(accepted, 6)
                    && unique(accepted)
                    && accepted.iter().all(|s| choices.contains(s))
            }
            Self::AdultReviewed { prompt_id } => label(prompt_id),
        };
        valid.then_some(()).ok_or(Error::InvalidTask)
    }

    /// Access accommodations must be selected by an approved representation adapter.
    /// A visual-only representation cannot silently certify a nonvisual interaction.
    pub fn validate_representation(&self, representation: &str) -> Result<(), Error> {
        let allowed = match self {
            Self::NumberSequence { .. } => &["text_choices", "labelled_objects"][..],
            Self::MissingQuantity { .. } => &["countable_planks", "number_buttons"][..],
            Self::EqualSharing { .. } => &["countable_objects", "group_buttons"][..],
            Self::Pattern { .. } | Self::Sort { .. } => &["labelled_objects", "text_choices"][..],
            Self::Position { .. } | Self::Measure { .. } => &["labelled_scene", "text_choices"][..],
            Self::Word { .. } => &["letter_tiles", "keyboard_tiles"][..],
            Self::StorySequence { .. } | Self::MessageChoice { .. } => {
                &["text_choices", "labelled_story_cards"][..]
            }
            Self::AdultReviewed { .. } => &["adult_observation"][..],
        };
        allowed
            .contains(&representation)
            .then_some(())
            .ok_or(Error::UnsupportedRepresentation)
    }
}

fn word_can_be_built(tiles: &[String], word: &str) -> bool {
    let mut remaining = tiles.to_vec();
    for c in word.chars() {
        let Some(index) = remaining.iter().position(|tile| tile == &c.to_string()) else {
            return false;
        };
        remaining.remove(index);
    }
    !word.is_empty()
}

/// Support is loaded from the server encounter, never supplied by the browser answer.
pub fn assess(task: &Task, answer: &Answer, support: Support) -> Result<Outcome, Error> {
    task.validate()?;
    let correct = match (task, answer) {
        (Task::NumberSequence { start, step, shown }, Answer::Number(value)) => {
            Some(*value == start + step * shown)
        }
        (Task::MissingQuantity { total, given }, Answer::Number(value)) => {
            Some(*value == total - given)
        }
        (Task::EqualSharing { total, recipients }, Answer::Groups(groups)) => {
            if groups.len() != usize::from(*recipients) || groups.iter().any(|n| *n > 20) {
                return Err(Error::InvalidAnswer);
            }
            Some(groups.iter().all(|n| *n == total / recipients))
        }
        (Task::Pattern { unit, prefix_len }, Answer::Text(value)) if label(value) => {
            Some(value == &unit[usize::from(*prefix_len) % unit.len()])
        }
        (Task::Sort { attributes, target }, Answer::Indices(indices)) => {
            if !unique(indices) || indices.iter().any(|i| *i >= attributes.len()) {
                return Err(Error::InvalidAnswer);
            }
            Some(
                indices.len() == attributes.iter().filter(|a| *a == target).count()
                    && indices.iter().all(|i| &attributes[*i] == target),
            )
        }
        (Task::Position { relation, choices }, Answer::Text(value)) => {
            if !choices.contains(value) {
                return Err(Error::InvalidAnswer);
            }
            Some(value == relation)
        }
        (Task::Measure { lengths }, Answer::Indices(indices))
            if indices.len() == 1 && indices[0] < lengths.len() =>
        {
            Some(Some(&lengths[indices[0]]) == lengths.iter().max())
        }
        (
            Task::Word {
                tiles, accepted, ..
            },
            Answer::Tiles(indices),
        ) => {
            if !unique(indices) || indices.len() > 12 || indices.iter().any(|i| *i >= tiles.len()) {
                return Err(Error::InvalidAnswer);
            }
            let assembled = indices
                .iter()
                .map(|i| tiles[*i].as_str())
                .collect::<String>();
            Some(accepted.contains(&assembled))
        }
        (Task::StorySequence { event_ids }, Answer::EventIds(answer_ids)) => {
            if answer_ids.len() != event_ids.len()
                || !unique(answer_ids)
                || answer_ids.iter().any(|id| !event_ids.contains(id))
            {
                return Err(Error::InvalidAnswer);
            }
            Some(answer_ids == event_ids)
        }
        (
            Task::MessageChoice {
                choices, accepted, ..
            },
            Answer::Text(value),
        ) => {
            if !choices.contains(value) {
                return Err(Error::InvalidAnswer);
            }
            Some(accepted.contains(value))
        }
        (Task::AdultReviewed { .. }, Answer::AdultResponse) => None,
        _ => return Err(Error::InvalidAnswer),
    };
    let reading = matches!(
        task,
        Task::Word {
            independently_read: true,
            ..
        } | Task::MessageChoice {
            independently_read: true,
            ..
        }
    );
    let classification = if correct.is_none() {
        Classification::Unassessed
    } else if support.hint_count > 0 || support.worked_example || (reading && support.read_aloud) {
        Classification::Supported
    } else {
        Classification::Independent
    };
    Ok(Outcome {
        correct,
        classification,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Family {
    Quantity,
    Sharing,
    PatternSort,
    SpatialMeasurement,
    PhonicsWord,
    StoryMessage,
}

/// Bounded authored variants; a recorded seed gives deterministic replay.
pub fn generate(family: Family, seed: u64) -> Task {
    let task = match family {
        Family::Quantity => {
            let total = 4 + (seed % 7) as u8;
            Task::MissingQuantity {
                total,
                given: (seed % u64::from(total)) as u8,
            }
        }
        Family::Sharing => {
            let recipients = 2 + (seed % 3) as u8;
            Task::EqualSharing {
                total: recipients * (1 + (seed % 3) as u8),
                recipients,
            }
        }
        Family::PatternSort => {
            if seed.is_multiple_of(2) {
                Task::Pattern {
                    unit: vec!["leaf".into(), "star".into()],
                    prefix_len: 4 + (seed % 3) as u8,
                }
            } else {
                Task::Sort {
                    attributes: vec!["round".into(), "square".into(), "round".into()],
                    target: "round".into(),
                }
            }
        }
        Family::SpatialMeasurement => {
            if seed.is_multiple_of(2) {
                Task::Position {
                    relation: "beside".into(),
                    choices: vec!["above".into(), "beside".into(), "inside".into()],
                }
            } else {
                Task::Measure {
                    lengths: vec![3, 5 + (seed % 3) as u8, 2],
                }
            }
        }
        Family::PhonicsWord => {
            let word = ["cat", "sun", "map"][(seed % 3) as usize];
            Task::Word {
                tiles: word.chars().map(|c| c.to_string()).collect(),
                accepted: vec![word.into()],
                independently_read: true,
            }
        }
        Family::StoryMessage => {
            if seed.is_multiple_of(2) {
                Task::StorySequence {
                    event_ids: vec!["find_seed".into(), "plant_seed".into(), "water_seed".into()],
                }
            } else {
                Task::MessageChoice {
                    choices: vec!["Pip needs a cup.".into(), "Pip needs a hat.".into()],
                    accepted: vec!["Pip needs a cup.".into()],
                    independently_read: true,
                }
            }
        }
    };
    debug_assert!(task.validate().is_ok());
    task
}

#[cfg(test)]
mod tests;

pub mod wire;
