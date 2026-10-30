//! Shared bounded plan verification plus offline corpus checks against authored mechanics.
use super::{authored_plan, build_plan_input, evaluation};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
use wonderwilds_domain::story::{Action, Command, Mission, MissionStatus, State};

type Result<T> = std::result::Result<T, String>;
pub const VERSION: &str = "mission-plan-eval-1";
const MAX_NODES: usize = 10_000;
const MAX_DEPTH: usize = 40;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scenario {
    pub version: String,
    pub source: evaluation::Scenario,
    pub input: Value,
    pub reference: Value,
    pub completed_condition_ids: Vec<String>,
}

fn project(source: evaluation::Scenario, state: &State) -> Result<Scenario> {
    Ok(Scenario {
        version: VERSION.into(),
        source,
        input: build_plan_input(state).map_err(|e| format!("{e:?}"))?,
        reference: authored_plan(state).map_err(|e| format!("{e:?}"))?,
        completed_condition_ids: state
            .milestones()
            .iter()
            .filter(|m| m.complete)
            .map(|m| m.condition_id.clone())
            .collect(),
    })
}

/// Two contrasting complete journeys, supply/repaired bridge edge cases and one
/// partial/resumed situation per mission. This is not an unseen mission holdout.
pub fn corpus() -> Result<Vec<Scenario>> {
    let mut seen = BTreeSet::new();
    let mut prompts = BTreeSet::new();
    let mut rows = vec![];
    for source in evaluation::corpus()? {
        let state = evaluation::validate_scenario(&source)?;
        if state.mission_status() != MissionStatus::Active {
            continue;
        }
        let family = source.family_id.as_str();
        let partial = state.milestones().iter().any(|m| m.complete);
        let selected = match family {
            "bank-series" => true,
            "far-gathering-woods-stars" => !partial,
            "supplied-series" | "repaired-series" => state.mission() == Mission::Bridge,
            _ => false,
        };
        if !selected {
            continue;
        }
        let key = (family.to_string(), state.mission_id().to_string(), partial);
        if seen.insert(key) {
            let row = project(source, &state)?;
            // Different later branches can share the exact same planning context.
            if prompts.insert(json!([row.input, row.completed_condition_ids]).to_string()) {
                rows.push(row);
            }
        }
    }
    Ok(rows)
}

pub fn validate_scenario(row: &Scenario) -> Result<State> {
    let state = evaluation::validate_scenario(&row.source)?;
    let expected = project(row.source.clone(), &state)?;
    if row.version != VERSION
        || row.input != expected.input
        || row.reference != expected.reference
        || row.completed_condition_ids != expected.completed_condition_ids
        || state.mission_status() != MissionStatus::Active
    {
        return Err("Planning scenario differs from replayed active runtime state".into());
    }
    Ok(state)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grade {
    pub scenario_id: String,
    pub candidate_id: String,
    pub passed: bool,
    pub failures: Vec<String>,
    pub explored_states: usize,
    pub witness: Vec<Action>,
}

fn done(state: &State, id: &str) -> bool {
    state
        .milestones()
        .iter()
        .any(|m| m.condition_id == id && m.complete)
}

fn advance(state: &State, ids: &[String], mut cursor: usize) -> Option<usize> {
    while cursor < ids.len() && done(state, &ids[cursor]) {
        cursor += 1;
    }
    // Conditions already satisfied at the start are excluded by the caller.
    // A later milestone cannot first complete before the current one.
    if ids[cursor..].iter().any(|id| done(state, id)) {
        None
    } else {
        Some(cursor)
    }
}

fn key(state: &State, cursor: usize) -> String {
    let mut projection = state.checkpoint()["state"].clone();
    let fields = projection.as_object_mut().unwrap();
    fields.remove("revision");
    fields.remove("events");
    fields.remove("history");
    fields.remove("receipts");
    json!([projection, cursor]).to_string()
}

pub fn grade(row: &Scenario, candidate: &evaluation::Candidate) -> Result<Grade> {
    let state = validate_scenario(row)?;
    if candidate.scenario_id != row.source.scenario_id {
        return Err("Wrong scenario binding".into());
    }
    let mut result = assess(&state, &candidate.output, None)?;
    result.scenario_id = candidate.scenario_id.clone();
    result.candidate_id = candidate.candidate_id.clone();
    Ok(result)
}

/// Shared mechanical validator for offline evaluation and live acceptance.
/// A live deadline fails closed; it never turns an incomplete search into a pass.
pub fn assess(
    state: &State,
    output: &Value,
    deadline: Option<std::time::Instant>,
) -> Result<Grade> {
    if state.mission_status() != MissionStatus::Active {
        return Err("Plan requires an active mission".into());
    }
    let input = build_plan_input(state).map_err(|e| format!("{e:?}"))?;
    let mut result = Grade {
        scenario_id: String::new(),
        candidate_id: String::new(),
        passed: false,
        failures: vec![],
        explored_states: 0,
        witness: vec![],
    };
    if let Err(error) = wonderwilds_contracts::validate("StoryMissionPlan", output) {
        result.failures.push(format!("schema: {error}"));
        return Ok(result);
    }
    if output["opportunityId"] != input["opportunities"][0]["id"] {
        result.failures.push("unknown_opportunity".into());
    }
    let characters: BTreeSet<_> = input["area"]["characters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    let mut participants = BTreeSet::new();
    for id in output["narrative"]["participantIds"].as_array().unwrap() {
        let id = id.as_str().unwrap();
        if !characters.contains(id) || !participants.insert(id) {
            result
                .failures
                .push("unknown_or_duplicate_participant".into());
        }
    }
    let mut unique = BTreeSet::new();
    let mut pending = vec![];
    for m in output["milestones"].as_array().unwrap() {
        let id = m["conditionId"].as_str().unwrap();
        if !unique.insert(id) {
            result.failures.push("duplicate_condition".into());
        }
        if !state.milestones().iter().any(|m| m.condition_id == id) {
            result.failures.push(format!("unavailable_condition: {id}"));
        }
        if !done(state, id) {
            pending.push(id.to_owned());
        }
    }
    // Require explicit coverage of the opportunity's actual terminal conditions.
    let terminal: &[&str] = match state.mission() {
        Mission::Bridge => &["crossed_with_pip"],
        Mission::Picnic => &["picnic_shared"],
        Mission::WelcomeLights => &["near_bank_lit", "far_bank_lit"],
        Mission::HummingHollow => &["hollow_shared"],
        Mission::CosyCorner => &["corner_shared"],
        Mission::BramblesVisit => &["bramble_visit_shared"],
        Mission::MoonlightGathering => &["gathering_shared"],
    };
    if terminal
        .iter()
        .any(|id| !done(state, id) && !unique.contains(id))
    {
        result.failures.push("missing_outcome_condition".into());
    }
    if !result.failures.is_empty() {
        return Ok(result);
    }
    let initial = state.clone();
    let mut visited = BTreeSet::from([key(state, 0)]);
    let mut queue = VecDeque::from([(state.clone(), 0, Vec::<Action>::new())]);
    let mut bounded = false;
    while let Some((state, cursor, path)) = queue.pop_front() {
        if deadline.is_some_and(|limit| std::time::Instant::now() >= limit) {
            bounded = true;
            break;
        }
        result.explored_states += 1;
        if cursor == pending.len() && state.mission_status() == MissionStatus::Completed {
            let mut replay = initial.clone();
            for (index, action) in path.iter().enumerate() {
                replay
                    .apply(&Command {
                        command_id: format!("witness-{index}-{}", replay.revision()),
                        expected_revision: replay.revision(),
                        action: action.clone(),
                    })
                    .map_err(|e| format!("Witness replay: {e:?}"))?;
            }
            if replay.compact().durable_checkpoint() != state.compact().durable_checkpoint() {
                return Err("Witness replay mismatch".into());
            }
            result.passed = true;
            result.witness = path;
            return Ok(result);
        }
        if path.len() >= MAX_DEPTH || state.command_budget_exhausted() {
            bounded = true;
            continue;
        }
        if result.explored_states >= MAX_NODES {
            bounded = true;
            break;
        }
        for action in state.available_actions() {
            // Neighbourhood detours cannot advance any Pip-series condition.
            // Keep the proof within the capability graph supplied to the planner.
            if matches!(&action, Action::Travel { destination_id } if !["bridge_bank", "workshop", "far_bank"].contains(&destination_id.as_str()))
            {
                continue;
            }
            if matches!(action, Action::AcceptMission | Action::CancelMission) {
                continue;
            }
            let mut next = state.clone();
            let command_id = (state.revision()..=state.revision() + 256)
                .map(|n| format!("planning-eval-{n}"))
                .find(|id| state.checkpoint()["receipts"].get(id).is_none())
                .unwrap();
            next.apply(&Command {
                command_id,
                expected_revision: state.revision(),
                action: action.clone(),
            })
            .map_err(|e| format!("Available action rejected: {e:?}"))?;
            let Some(next_cursor) = advance(&next, &pending, cursor) else {
                continue;
            };
            if visited.insert(key(&next, next_cursor)) {
                let mut next_path = path.clone();
                next_path.push(action);
                queue.push_back((next, next_cursor, next_path));
            }
        }
    }
    result.failures.push(
        if bounded {
            "search_bound_unproven"
        } else {
            "no_ordered_completion_path"
        }
        .into(),
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidate(row: &Scenario, output: Value) -> evaluation::Candidate {
        evaluation::Candidate {
            scenario_id: row.source.scenario_id.clone(),
            candidate_id: "test".into(),
            output,
            generator: None,
        }
    }
    #[test]
    fn references_have_replayable_completion_witnesses() {
        let rows = corpus().unwrap();
        assert_eq!(rows.len(), 18);
        for row in rows {
            let g = grade(&row, &candidate(&row, row.reference.clone())).unwrap();
            assert!(g.passed, "{}: {:?}", g.scenario_id, g.failures);
        }
    }
    #[test]
    fn accepts_either_lantern_order_and_rejects_duplicates_and_absent_characters() {
        let rows = corpus().unwrap();
        let row = rows
            .iter()
            .find(|r| {
                r.input["opportunities"][0]["id"] == "welcome_lights"
                    && r.completed_condition_ids.is_empty()
            })
            .unwrap();
        let mut output = row.reference.clone();
        output["milestones"].as_array_mut().unwrap().swap(2, 3);
        assert!(grade(row, &candidate(row, output)).unwrap().passed);
        let mut output = row.reference.clone();
        output["narrative"]["participantIds"] = json!(["unknown_character"]);
        assert!(!grade(row, &candidate(row, output)).unwrap().passed);
        let mut output = row.reference.clone();
        let duplicate = output["milestones"][0].clone();
        output["milestones"].as_array_mut().unwrap().push(duplicate);
        assert!(
            grade(row, &candidate(row, output))
                .unwrap()
                .failures
                .contains(&"duplicate_condition".into())
        );
    }

    #[test]
    fn rejects_invalid_bindings_missing_outcome_and_impossible_order() {
        let row = corpus().unwrap().remove(0);
        for (field, value) in [("opportunityId", json!("invented")), ("reward", json!(900))] {
            let mut output = row.reference.clone();
            output[field] = value;
            assert!(!grade(&row, &candidate(&row, output)).unwrap().passed);
        }
        let mut output = row.reference.clone();
        output["milestones"].as_array_mut().unwrap().pop();
        assert!(
            grade(&row, &candidate(&row, output))
                .unwrap()
                .failures
                .contains(&"missing_outcome_condition".into())
        );
        let mut output = row.reference.clone();
        output["milestones"].as_array_mut().unwrap().swap(0, 1);
        assert_eq!(
            grade(&row, &candidate(&row, output)).unwrap().failures,
            ["no_ordered_completion_path"]
        );
        let mut changed = row.clone();
        changed.input["context"]["locationId"] = json!("far_bank");
        assert!(validate_scenario(&changed).is_err());
    }
}
