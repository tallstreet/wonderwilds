//! Validated, state-bound mission briefs. Mechanics and completion remain in Rust.
use super::*;
use std::time::{Duration, Instant};
use wonderwilds_domain::story::MissionStatus;

pub trait PlanProvider: Send + Sync {
    fn propose(&self, input: &Value, completed: &[String]) -> Result<Value, director::Error>;
}

/// Constructed only after a bounded real-domain completion search. The checkpoint
/// binding prevents applying a valid plan to a different acceptance or revision.
pub struct ValidatedPlan {
    checkpoint: Value,
    plan: Value,
}
impl ValidatedPlan {
    pub fn new(state: &State, plan: Value) -> Result<Self, Error> {
        let grade = director::planning_evaluation::assess(
            state,
            &plan,
            Some(Instant::now() + Duration::from_millis(500)),
        )
        .map_err(|_| Error::InvalidInput)?;
        if !grade.passed {
            return Err(Error::InvalidInput);
        }
        Ok(Self {
            checkpoint: state.checkpoint(),
            plan,
        })
    }
    pub fn for_state(&self, state: &State) -> Result<&Value, Error> {
        if self.checkpoint != state.checkpoint() {
            return Err(Error::RevisionConflict);
        }
        Ok(&self.plan)
    }
}

/// Only a newly accepted mission gets a plan. Pause/resume retains the saved brief.
pub fn acceptance(state: &State, request: &Value) -> Result<Option<State>, Error> {
    let (next, _) = super::execute(state, request)?;
    if matches!(
        state.mission_status(),
        MissionStatus::Offered | MissionStatus::Completed
    ) && next.mission_status() == MissionStatus::Active
    {
        Ok(Some(next))
    } else {
        Ok(None)
    }
}

/// Re-project saved labels from authoritative condition receipts, never model claims.
pub fn project(state: &State, plan: &Value, snapshot: &mut Value) -> Result<(), Error> {
    wire::validate("StoryMissionPlan", plan).map_err(|_| Error::Unavailable)?;
    if plan["opportunityId"] != state.mission_id() {
        return Err(Error::Unavailable);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut completed = vec![];
    let mut pending = vec![];
    for item in plan["milestones"].as_array().unwrap() {
        let id = item["conditionId"].as_str().unwrap();
        if !seen.insert(id) {
            return Err(Error::Unavailable);
        }
        let milestone = state
            .milestones()
            .iter()
            .find(|m| m.condition_id == id)
            .ok_or(Error::Unavailable)?;
        if milestone.complete {
            completed.push(item["label"].clone());
        } else {
            pending.push(item["label"].clone());
        }
    }
    if snapshot["commandBudgetExhausted"] == true {
        return Ok(());
    }
    let mission = &mut snapshot["sceneInput"]["mission"];
    mission["goal"] = plan["goal"].clone();
    mission["narrative"] = plan["narrative"].clone();
    mission["completed"] = json!(completed);
    mission["current"] = json!(pending.first());
    mission["remaining"] = json!(pending.iter().skip(1).collect::<Vec<_>>());
    wire::validate("StorySnapshot", snapshot).map_err(|_| Error::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation_is_state_bound_and_deadlines_never_pass() {
        let mut state = State::new();
        state
            .apply(&Command {
                command_id: "accept".into(),
                expected_revision: 0,
                action: Action::AcceptMission,
            })
            .unwrap();
        let plan = director::authored_plan(&state).unwrap();
        let validated = ValidatedPlan::new(&state, plan.clone()).unwrap();
        let expired =
            director::planning_evaluation::assess(&state, &plan, Some(Instant::now())).unwrap();
        assert!(!expired.passed);
        assert_eq!(expired.failures, vec!["search_bound_unproven"]);
        state
            .apply(&Command {
                command_id: "travel".into(),
                expected_revision: 1,
                action: Action::Travel {
                    destination_id: "workshop".into(),
                },
            })
            .unwrap();
        assert_eq!(validated.for_state(&state), Err(Error::RevisionConflict));
        let mut snapshot = super::super::snapshot(&state).unwrap();
        let mut invalid = plan;
        invalid["milestones"][0]["conditionId"] = json!("invented");
        assert_eq!(
            project(&state, &invalid, &mut snapshot),
            Err(Error::Unavailable)
        );
    }
}
