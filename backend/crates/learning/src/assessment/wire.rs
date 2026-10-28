//! Closed conversion between authored server tasks and the versioned public protocol.
use super::*;
use serde_json::{Value, json};
use wonderwilds_contracts::{Answer as WireAnswer, EncounterInstance, Parameters};

pub const CONTENT_VERSION: &str = "foundation-draft-1";

/// Public prompt registry contains no answer keys; adapters must reject unknown IDs.
pub fn prompt(id: &str) -> Option<String> {
    let registry: Value = serde_json::from_str(include_str!(
        "../../../../../content/activities/prompts-v1.json"
    ))
    .ok()?;
    registry
        .get(id)?
        .get("instruction")?
        .as_str()
        .map(str::to_owned)
}

fn str_array(values: &[String]) -> Value {
    json!(values)
}

/// Build only reviewed-in-code authored *draft* tasks. Educator release approval is separate.
/// Seeds are recorded by the API so replay never calls an RNG.
pub fn authored_encounter(
    skill_id: &str,
    seed: u64,
    id: &str,
) -> Result<(Task, EncounterInstance), Error> {
    let task = match skill_id {
        "M05" => Task::NumberSequence {
            start: 1 + (seed % 3) as u8,
            step: 1,
            shown: 3,
        },
        "M06" => generate(Family::Quantity, seed),
        "M08" => generate(Family::Sharing, seed),
        "M09" => generate(Family::PatternSort, seed - seed % 2),
        "M14" => generate(Family::PatternSort, seed | 1),
        "M11" => generate(Family::SpatialMeasurement, seed - seed % 2),
        "M12" => generate(Family::SpatialMeasurement, seed | 1),
        "E05" => Task::Word {
            tiles: vec!["s".into(), "m".into(), "t".into()],
            accepted: vec!["s".into()],
            independently_read: false,
        },
        "E06" | "E07" => generate(Family::PhonicsWord, seed),
        "E01" => Task::MessageChoice {
            choices: vec![
                "Put the cup beside the tree.".into(),
                "Put the cup inside the box.".into(),
            ],
            accepted: vec!["Put the cup beside the tree.".into()],
            independently_read: false,
        },
        "E10" | "E12" => Task::MessageChoice {
            choices: vec![
                "Pip needs a red cup.".into(),
                "Pip needs a blue hat.".into(),
            ],
            accepted: vec!["Pip needs a red cup.".into()],
            independently_read: true,
        },
        "E11" => generate(Family::StoryMessage, seed - seed % 2),
        _ => return Err(Error::InvalidTask),
    };
    task.validate_skill(skill_id)?;
    let (template, params) = match &task {
        Task::MissingQuantity { total, given } => {
            ("missing_quantity", json!({"total":total,"given":given}))
        }
        Task::EqualSharing { total, recipients } => (
            "equal_sharing",
            json!({"total":total,"recipients":recipients}),
        ),
        Task::NumberSequence { start, step, shown } => {
            let next = start + step * shown;
            let mut objects = (0..*shown).map(|i| json!({"id":format!("shown-{i}"),"label":(start + step*i).to_string(),"attribute":"shown"})).collect::<Vec<_>>();
            for value in [next - 1, next, next + 1] {
                objects.push(json!({"id":format!("number-{value}"),"label":value.to_string(),"attribute":"choice"}));
            }
            (
                "pattern_sort",
                json!({"objects":objects,"prefixLength":shown,"instructionId":"count-next"}),
            )
        }
        Task::Pattern { unit, prefix_len } => {
            let mut objects = (0..*prefix_len).map(|i| json!({"id":format!("shown-{i}"),"label":unit[usize::from(i)%unit.len()],"attribute":"shown"})).collect::<Vec<_>>();
            for (i, value) in unit.iter().enumerate() {
                objects
                    .push(json!({"id":format!("choice-{i}"),"label":value,"attribute":"choice"}));
            }
            (
                "pattern_sort",
                json!({"objects":objects,"unitLength":unit.len(),"prefixLength":prefix_len,"instructionId":"pattern-next"}),
            )
        }
        Task::Sort { attributes, .. } => (
            "pattern_sort",
            json!({"objects":attributes.iter().enumerate().map(|(i,a)|json!({"id":format!("object-{i}"),"label":format!("{} object {}",a,i+1),"attribute":a})).collect::<Vec<_>>(),"instructionId":"sort-round"}),
        ),
        Task::Position { choices, .. } => (
            "spatial_measurement",
            json!({"objects":choices.iter().enumerate().map(|(i,c)|json!({"id":format!("position-{i}"),"label":c})).collect::<Vec<_>>(),"instructionId":"place-beside"}),
        ),
        Task::Measure { lengths } => (
            "spatial_measurement",
            json!({"objects":lengths.iter().enumerate().map(|(i,n)|json!({"id":format!("rope-{i}"),"label":format!("rope {}",i+1),"length":n})).collect::<Vec<_>>(),"instructionId":"longest-rope"}),
        ),
        Task::Word { tiles, .. } => (
            "phonics_word_building",
            json!({"tiles":str_array(tiles),"promptId":if skill_id=="E05" {"sound-s"} else {match seed%3 {0=>"word-cat",1=>"word-sun",_=>"word-map"}}}),
        ),
        Task::StorySequence { event_ids } => {
            // Presentation order differs from canonical solution. IDs carry identity, never sequence numbers.
            let mut cards=event_ids.iter().map(|id| json!({"id":id,"text":match id.as_str(){"find_seed"=>"Pip finds a seed.","plant_seed"=>"Pip plants the seed.",_=>"Pip waters the seed."}})).collect::<Vec<_>>();
            cards.rotate_left(1);
            (
                "story_message",
                json!({"cards":cards,"promptId":"seed-order"}),
            )
        }
        Task::MessageChoice { choices, .. } => (
            "story_message",
            json!({"cards":choices.iter().enumerate().map(|(i,c)|json!({"id":format!("message-{i}"),"text":c})).collect::<Vec<_>>(),"promptId":if skill_id=="E01" {"listen-direction"}else{"read-red-cup"}}),
        ),
        Task::AdultReviewed { .. } => return Err(Error::InvalidTask),
    };
    if let Some(prompt_id) = params
        .get("promptId")
        .or_else(|| params.get("instructionId"))
        && prompt(prompt_id.as_str().ok_or(Error::InvalidTask)?).is_none()
    {
        return Err(Error::InvalidTask);
    }
    let value = json!({"schemaVersion":"1.0","id":id,"templateId":template,"skillId":skill_id,"contentVersion":CONTENT_VERSION,"parameters":params,"permittedActions":["attempt","hint","worked_example","abandon"],"support":{"hintCount":0,"workedExample":false,"readAloud":false}});
    wonderwilds_contracts::validate("EncounterInstance", &value).map_err(|_| Error::InvalidTask)?;
    let encounter = serde_json::from_value(value).map_err(|_| Error::InvalidTask)?;
    Ok((task, encounter))
}

/// Convert a wire answer using the server's original encounter. Never trust client objects.
pub fn assess_wire(
    task: &Task,
    encounter: &EncounterInstance,
    answer: &WireAnswer,
) -> Result<Outcome, Error> {
    task.validate_skill(&encounter.skill_id)?;
    let value = serde_json::to_value(answer).map_err(|_| Error::InvalidAnswer)?;
    wonderwilds_contracts::validate("Answer", &value).map_err(|_| Error::InvalidAnswer)?;
    let parameters = serde_json::to_value(&encounter.parameters).map_err(|_| Error::InvalidTask)?;
    let ids = value.get("choiceIds").and_then(Value::as_array);
    let choice_ids = || -> Result<Vec<&str>, Error> {
        let values = ids
            .ok_or(Error::InvalidAnswer)?
            .iter()
            .map(|id| id.as_str().ok_or(Error::InvalidAnswer))
            .collect::<Result<Vec<_>, _>>()?;
        if !unique(&values) {
            return Err(Error::InvalidAnswer);
        }
        Ok(values)
    };
    let indices = |field: &str| -> Result<Vec<usize>, Error> {
        choice_ids()?
            .iter()
            .map(|id| {
                parameters[field]
                    .as_array()
                    .ok_or(Error::InvalidTask)?
                    .iter()
                    .position(|item| item["id"].as_str() == Some(id))
                    .ok_or(Error::InvalidAnswer)
            })
            .collect()
    };
    let internal = match task {
        Task::MissingQuantity { .. } => Answer::Number(
            u8::try_from(value["value"].as_u64().ok_or(Error::InvalidAnswer)?)
                .map_err(|_| Error::InvalidAnswer)?,
        ),
        Task::EqualSharing { .. } => Answer::Groups(
            value["groups"]
                .as_array()
                .ok_or(Error::InvalidAnswer)?
                .iter()
                .map(|n| {
                    u8::try_from(n.as_u64().ok_or(Error::InvalidAnswer)?)
                        .map_err(|_| Error::InvalidAnswer)
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Task::NumberSequence { .. } | Task::Pattern { .. } => {
            let chosen = indices("objects")?;
            if chosen.len() != 1 {
                return Err(Error::InvalidAnswer);
            }
            let object = &parameters["objects"][chosen[0]];
            if object["attribute"] != "choice" {
                return Err(Error::InvalidAnswer);
            }
            let text = object["label"].as_str().ok_or(Error::InvalidTask)?;
            if matches!(task, Task::NumberSequence { .. }) {
                Answer::Number(text.parse().map_err(|_| Error::InvalidAnswer)?)
            } else {
                Answer::Text(text.into())
            }
        }
        Task::Sort { .. } | Task::Measure { .. } => Answer::Indices(indices("objects")?),
        Task::Position { .. } => {
            let chosen = indices("objects")?;
            if chosen.len() != 1 {
                return Err(Error::InvalidAnswer);
            }
            Answer::Text(
                parameters["objects"][chosen[0]]["label"]
                    .as_str()
                    .ok_or(Error::InvalidTask)?
                    .into(),
            )
        }
        Task::Word { tiles, .. } => {
            let selected = choice_ids()?
                .iter()
                .map(|id| id.parse::<usize>().map_err(|_| Error::InvalidAnswer))
                .collect::<Result<Vec<_>, _>>()?;
            if selected.iter().any(|i| *i >= tiles.len()) {
                return Err(Error::InvalidAnswer);
            }
            Answer::Tiles(selected)
        }
        Task::StorySequence { .. } => {
            indices("cards")?;
            Answer::EventIds(choice_ids()?.iter().map(|s| s.to_string()).collect())
        }
        Task::MessageChoice { .. } => {
            let chosen = indices("cards")?;
            if chosen.len() != 1 {
                return Err(Error::InvalidAnswer);
            }
            Answer::Text(
                parameters["cards"][chosen[0]]["text"]
                    .as_str()
                    .ok_or(Error::InvalidTask)?
                    .into(),
            )
        }
        Task::AdultReviewed { .. } => return Err(Error::InvalidAnswer),
    };
    assess(
        task,
        &internal,
        Support {
            hint_count: u8::try_from(encounter.support.hint_count)
                .map_err(|_| Error::InvalidTask)?,
            worked_example: encounter.support.worked_example,
            read_aloud: encounter.support.read_aloud,
            motor_access: false,
        },
    )
}

pub fn authored_parameters(skill_id: &str, seed: u64) -> Result<(Task, Parameters), Error> {
    let (task, encounter) = authored_encounter(skill_id, seed, "authored-proposal")?;
    Ok((task, encounter.parameters))
}

#[cfg(test)]
mod tests;
