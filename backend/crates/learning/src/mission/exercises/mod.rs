//! Current mission questions. The authored bank is internal; only its bounded public projection leaves Rust.
use super::*;
use serde_json::{Value, json};
use std::sync::OnceLock;
mod marking;

pub const VERSION: &str = "mission-exercises-draft-1";
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item {
    pub title: String,
    pub prompt: String,
    pub representation: String,
    pub stimuli: Vec<Value>,
    pub interaction: Value,
    pub answer: Option<Value>,
    pub hints: Vec<String>,
    pub explanation: String,
    pub semantic_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Exercise {
    id: String,
    items: Vec<Item>,
    facet: String,
    difficulty_rungs: Vec<String>,
    assessment_mode: String,
    limitations: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Skill {
    skill_id: String,
    exercises: Vec<Exercise>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Bank {
    schema_version: String,
    review_status: String,
    release_enabled: bool,
    skills: Vec<Skill>,
}
fn bank() -> &'static Bank {
    static BANK: OnceLock<Bank> = OnceLock::new();
    BANK.get_or_init(|| {
        let bank: Bank = serde_json::from_str(include_str!(
            "../../../../../../content/activities/skill-exercises-v1.json"
        ))
        .expect("validated authored exercise bank");
        assert_eq!(bank.schema_version, "1.0");
        assert_eq!(bank.review_status, "pending");
        assert!(!bank.release_enabled);
        assert!(
            bank.skills
                .iter()
                .flat_map(|s| &s.exercises)
                .all(|e| !e.facet.is_empty()
                    && !e.difficulty_rungs.is_empty()
                    && ["automatic", "human_review"].contains(&e.assessment_mode.as_str())
                    && (e.assessment_mode != "human_review" || !e.limitations.is_empty()))
        );
        bank
    })
}
pub fn available(skill: &str) -> bool {
    bank().skills.iter().any(|s| s.skill_id == skill)
}
pub fn activity(skill: &str, seed: u64, id: &str) -> Result<MissionActivity, MissionError> {
    let entry = bank()
        .skills
        .iter()
        .find(|s| s.skill_id == skill)
        .ok_or(MissionError::UnsupportedSkill)?;
    let exercise = &entry.exercises[seed as usize % entry.exercises.len()];
    let mut item =
        exercise.items[(seed as usize / entry.exercises.len()) % exercise.items.len()].clone();
    let mut interaction = item.interaction.clone();
    if let Some(options) = interaction.get_mut("options").and_then(Value::as_array_mut) {
        let len = options.len();
        options.rotate_left((seed as usize + seed as usize / entry.exercises.len()) % len);
        let mapping: BTreeMap<String, String> = options
            .iter()
            .enumerate()
            .map(|(i, o)| (o["id"].as_str().unwrap().into(), format!("option-{i}")))
            .collect();
        if let Some(answer) = &mut item.answer {
            if let Some(id) = answer.get_mut("choiceId") {
                *id = json!(mapping[id.as_str().unwrap()]);
            }
            if let Some(ids) = answer.get_mut("choiceIds").and_then(Value::as_array_mut) {
                for id in ids {
                    *id = json!(mapping[id.as_str().unwrap()]);
                }
            }
        }
        for option in options {
            option["id"] = json!(mapping[option["id"].as_str().unwrap()]);
        }
    }
    if interaction["kind"] == "order" {
        let items = interaction["items"]
            .as_array_mut()
            .ok_or(MissionError::InvalidState)?;
        let len = items.len();
        items.rotate_left(seed as usize % len);
        let mapping: BTreeMap<String, String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| (item["id"].as_str().unwrap().into(), format!("tile-{i}")))
            .collect();
        if let Some(answer) = &mut item.answer {
            for id in answer["itemIds"].as_array_mut().unwrap() {
                *id = json!(mapping[id.as_str().unwrap()]);
            }
        }
        for item in items {
            item["id"] = json!(mapping[item["id"].as_str().unwrap()]);
        }
    }
    item.interaction = interaction.clone();
    let node = CurriculumGraph::load()
        .map_err(|_| MissionError::InvalidState)?
        .get(skill)
        .ok_or(MissionError::UnsupportedSkill)?
        .clone();
    let task = PublicTask::Question {
        exercise_id: exercise.id.clone(),
        interaction,
        stimuli: item.stimuli.clone(),
    };
    let view = MissionActivityView {
        schema_version: "1.0".into(),
        id: id.into(),
        skill_id: skill.into(),
        content_version: VERSION.into(),
        variant_id: format!("{skill}-{}-{}-{seed}", VERSION, exercise.id),
        context_id: "authored-practice".into(),
        representation: item.representation.clone(),
        presentation: MissionPresentation {
            title: item.title.clone(),
            objective: node.objective,
            lesson_text: "Try the question, or ask for help.".into(),
            prompt: item.prompt.clone(),
            context: "Pip has a little challenge for you.".into(),
            representation: item.representation.clone(),
        },
        task,
    };
    let mut wire = serde_json::to_value(&view).map_err(|_| MissionError::InvalidState)?;
    wire["support"] = json!({"hintCount":0,"workedExample":false,"readAloud":false});
    wonderwilds_contracts::validate("MissionActivity", &wire).map_err(|_error| {
        #[cfg(test)]
        eprintln!("Question contract {skill}/{seed}: {_error}");
        MissionError::InvalidState
    })?;
    Ok(MissionActivity {
        view,
        key: AssessmentKey::Question(Box::new(item.clone())),
        canonical_fingerprint: format!(
            "{}:{}:{}",
            exercise.id, item.semantic_id, item.representation
        ),
    })
}
pub fn assess(item: &Item, answer: &MissionAnswer) -> Result<bool, MissionError> {
    marking::assess(item, answer)
}

#[cfg(test)]
mod tests;
