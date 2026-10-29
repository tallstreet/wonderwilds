//! Versioned wire models only: no gameplay decisions, browser types or credentials.
pub mod models;
pub use models::*;

use std::collections::BTreeMap;
use std::sync::OnceLock;

struct ContractCache {
    schema: serde_json::Value,
    validators: BTreeMap<String, OnceLock<Result<jsonschema::Validator, String>>>,
}
static CACHE: OnceLock<Result<ContractCache, String>> = OnceLock::new();

/// Validate bounds and closed-object semantics before deserializing a wire model.
/// Only checked-in definitions enter the cache; caller strings cannot grow it.
pub fn validate(contract: &str, value: &serde_json::Value) -> Result<(), String> {
    let cache = CACHE
        .get_or_init(|| {
            let schema: serde_json::Value =
                serde_json::from_str(include_str!("../schemas/v1.json"))
                    .map_err(|error| error.to_string())?;
            let definitions = schema["$defs"]
                .as_object()
                .ok_or("Missing contract definitions")?;
            let validators = definitions
                .keys()
                .map(|name| (name.clone(), OnceLock::new()))
                .collect();
            Ok(ContractCache { schema, validators })
        })
        .as_ref()
        .map_err(Clone::clone)?;
    let slot = cache.validators.get(contract).ok_or("Unknown contract")?;
    let validator = slot
        .get_or_init(|| {
            let mut schema = cache.schema.clone();
            schema["$ref"] = serde_json::Value::String(format!("#/$defs/{contract}"));
            jsonschema::options()
                .should_validate_formats(true)
                .build(&schema)
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)?;
    validator.validate(value).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_positive_and_negative_fixtures() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/cases.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let name = case["contract"].as_str().unwrap();
            assert_eq!(
                validate(name, &case["value"]).is_ok(),
                case["valid"].as_bool().unwrap(),
                "{}",
                case["name"]
            );
            if case["valid"].as_bool().unwrap() {
                macro_rules! round_trip {
                    ($ty:ty) => {{
                        let model: $ty = serde_json::from_value(case["value"].clone()).unwrap();
                        assert_eq!(serde_json::to_value(model).unwrap(), case["value"]);
                    }};
                }
                match name {
                    "GameCommand" => round_trip!(GameCommand),
                    "GameSnapshot" => round_trip!(GameSnapshot),
                    "EncounterInstance" => round_trip!(EncounterInstance),
                    "LearningEvidence" => round_trip!(LearningEvidence),
                    "DomainEvent" => round_trip!(DomainEvent),
                    "ContentManifest" => round_trip!(ContentManifest),
                    "DirectorInput" => round_trip!(DirectorInput),
                    "DirectorProposal" => round_trip!(DirectorProposal),
                    "SkillNode" => round_trip!(SkillNode),
                    "ApiError" => round_trip!(ApiError),
                    "TrainingBinding" => round_trip!(TrainingBinding),
                    "TrainingOffer" => round_trip!(TrainingOffer),
                    "CharacterProgression" => round_trip!(CharacterProgression),
                    "ExplorerProgression" => round_trip!(ExplorerProgression),
                    "NarrativeMemory" => round_trip!(NarrativeMemory),
                    "QuestDocument" => round_trip!(QuestDocument),
                    "PersonalQuestView" => round_trip!(PersonalQuestView),
                    "PersonalQuestBinding" => round_trip!(PersonalQuestBinding),
                    "QuestGenerationContext" => round_trip!(QuestGenerationContext),
                    "QuestGenerationProposal" => round_trip!(QuestGenerationProposal),
                    _ => panic!("Missing round trip for {name}"),
                }
            }
        }
    }
}
