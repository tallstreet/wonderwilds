use super::*;
fn invalid<T>() -> Result<T, MissionError> {
    Err(MissionError::InvalidAnswer)
}
fn ids(values: &Value, field: &str) -> BTreeSet<String> {
    values
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v[field].as_str().map(str::to_owned))
        .collect()
}
fn unique(values: &[String]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
/// Exact bounded decimal/fraction parser. Never evaluate expressions or round a child's value.
fn rational(text: &str) -> Option<(i128, i128)> {
    let text = text.trim();
    if text.len() > 32 || text.is_empty() {
        return None;
    }
    if let Some((a, b)) = text.split_once('/') {
        let a = integer(a)?;
        let b = integer(b)?;
        if b <= 0 {
            return None;
        }
        return Some((a, b));
    }
    if let Some((whole, tail)) = text.split_once('.') {
        if tail.is_empty() || tail.len() > 6 || !tail.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let negative = whole.starts_with('-');
        let whole = integer(whole)?;
        let denominator = 10_i128.pow(tail.len() as u32);
        let fraction = tail.parse::<i128>().ok()?;
        return Some((
            whole * denominator + if negative { -fraction } else { fraction },
            denominator,
        ));
    }
    Some((integer(text)?, 1))
}
fn integer(text: &str) -> Option<i128> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || digits.len() > 12 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}
pub(super) fn assess(item: &Item, answer: &MissionAnswer) -> Result<bool, MissionError> {
    let Some(expected) = &item.answer else {
        return invalid();
    };
    let value = serde_json::to_value(answer).map_err(|_| MissionError::InvalidAnswer)?;
    let interaction = &item.interaction;
    match (interaction["kind"].as_str().unwrap_or(""), answer) {
        ("single_choice", MissionAnswer::Choice { choice_id }) => {
            if !ids(&interaction["options"], "id").contains(choice_id) {
                return invalid();
            }
            Ok(value == *expected)
        }
        ("multi_choice", MissionAnswer::Choices { choice_ids }) => {
            let allowed = ids(&interaction["options"], "id");
            if choice_ids.is_empty()
                || !unique(choice_ids)
                || choice_ids.iter().any(|id| !allowed.contains(id))
            {
                return invalid();
            }
            Ok(choice_ids.iter().cloned().collect::<BTreeSet<_>>()
                == expected["choiceIds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect())
        }
        ("number_entry", MissionAnswer::Numbers { values }) => {
            let fields = interaction["fields"].as_array().unwrap();
            if fields.len() != values.len()
                || values
                    .iter()
                    .map(|v| &v.field_id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != values.len()
            {
                return invalid();
            }
            if fields.len() == 3
                && fields[1]["label"] == "Numerator"
                && fields[2]["label"] == "Denominator"
            {
                let read = |index: usize| -> Result<i128, MissionError> {
                    let field = &fields[index];
                    let value = values
                        .iter()
                        .find(|v| v.field_id == field["id"].as_str().unwrap())
                        .ok_or(MissionError::InvalidAnswer)?;
                    integer(value.value.trim()).ok_or(MissionError::InvalidAnswer)
                };
                let (whole, numerator, denominator) = (read(0)?, read(1)?, read(2)?);
                if denominator <= 0 {
                    return invalid();
                }
                if numerator < 0 || numerator >= denominator {
                    return Ok(false);
                }
                let expected = expected["values"]
                    .as_array()
                    .ok_or(MissionError::InvalidState)?;
                let parsed: Vec<_> = expected
                    .iter()
                    .map(|v| integer(v["value"].as_str().unwrap()).unwrap())
                    .collect();
                return Ok(whole == parsed[0] && numerator * parsed[2] == parsed[1] * denominator);
            }
            let mut correct = true;
            for field in fields {
                let value = values
                    .iter()
                    .find(|v| v.field_id == field["id"].as_str().unwrap())
                    .ok_or(MissionError::InvalidAnswer)?;
                let text = value.value.trim();
                match field["format"].as_str() {
                    Some("integer") if integer(text).is_none() => return invalid(),
                    Some("decimal") if text.contains('/') => return invalid(),
                    _ => {}
                }
                let (a, b) = rational(text).ok_or(MissionError::InvalidAnswer)?;
                let expected = expected["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["fieldId"] == field["id"])
                    .unwrap();
                let (c, d) = rational(expected["value"].as_str().unwrap())
                    .ok_or(MissionError::InvalidState)?;
                correct &= a * d == c * b;
            }
            Ok(correct)
        }
        ("text_entry", MissionAnswer::Text { value }) => {
            let text = value.trim();
            if text.is_empty() || text.chars().count() > 120 {
                return invalid();
            }
            Ok(text.to_lowercase() == expected["value"].as_str().unwrap().to_lowercase())
        }
        ("order", MissionAnswer::Order { item_ids }) => {
            let allowed = ids(&interaction["items"], "id");
            if !unique(item_ids) || item_ids.iter().cloned().collect::<BTreeSet<_>>() != allowed {
                return invalid();
            }
            let labels = |ids: &Value| -> Vec<String> {
                ids.as_array()
                    .unwrap()
                    .iter()
                    .map(|id| {
                        interaction["items"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|i| i["id"] == *id)
                            .unwrap()["label"]
                            .as_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect()
            };
            Ok(labels(&value["itemIds"]) == labels(&expected["itemIds"]))
        }
        ("assign", MissionAnswer::Assign { assignments }) => {
            let objects = ids(&interaction["items"], "id");
            let targets = ids(&interaction["targets"], "id");
            if assignments.len() != objects.len()
                || assignments
                    .iter()
                    .map(|v| v.object_id.clone())
                    .collect::<BTreeSet<_>>()
                    != objects
                || assignments.iter().any(|v| !targets.contains(&v.target_id))
            {
                return invalid();
            }
            if expected.get("rule").and_then(Value::as_str) == Some("equal") {
                let counts: Vec<_> = targets
                    .iter()
                    .map(|t| assignments.iter().filter(|a| &a.target_id == t).count())
                    .collect();
                return Ok(counts.windows(2).all(|w| w[0] == w[1]));
            }
            let expected: MissionAnswer =
                serde_json::from_value(expected.clone()).map_err(|_| MissionError::InvalidState)?;
            let grouped = |assignments: &[TargetAssignment]| {
                let mut counts = BTreeMap::new();
                for assignment in assignments {
                    let label = interaction["items"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|i| i["id"] == assignment.object_id)
                        .unwrap()["label"]
                        .as_str()
                        .unwrap()
                        .to_owned();
                    *counts
                        .entry((label, assignment.target_id.clone()))
                        .or_insert(0usize) += 1;
                }
                counts
            };
            if let MissionAnswer::Assign {
                assignments: expected,
            } = &expected
            {
                Ok(grouped(assignments) == grouped(expected))
            } else {
                invalid()
            }
        }
        _ => invalid(),
    }
}
