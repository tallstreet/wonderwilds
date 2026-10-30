//! Authored story application boundary, separate from assessed-game progress.
//! Client input is only a revision-bound choice or a presentation acknowledgement.
use crate::game::Error;
use serde_json::{Value, json};
use wonderwilds_contracts as wire;
use wonderwilds_director::story as director;
use wonderwilds_domain::story::{Action, Command, Location, Rejection, State};

pub mod http;
pub mod modal;
pub mod repository;

/// Only the woodcutter introduction currently creates a presented-history fact.
/// Repeated renders and acknowledgements of other authored lines do not consume
/// commands or imply that the child understood the text.
fn acknowledgeable_entries(state: &State) -> Vec<&'static str> {
    if state.location() == Location::Workshop
        && !state.met_woodcutter()
        && state.revision() < wonderwilds_domain::story::MAX_COMMANDS as u64
    {
        vec!["greeting"]
    } else {
        vec![]
    }
}

pub fn snapshot(state: &State) -> Result<Value, Error> {
    let exhausted = state.revision() >= wonderwilds_domain::story::MAX_COMMANDS as u64;
    let mut value = json!({
        "schemaVersion":"1.0",
        "contentVersion":wonderwilds_domain::story::CONTENT_VERSION,
        "revision":state.revision(),
        "locationId":state.location().as_str(),
        "commandBudgetExhausted":exhausted,
        "acknowledgeableEntryIds":acknowledgeable_entries(state)
    });
    if !exhausted {
        value["sceneInput"] = director::build_scene_input(state).map_err(|_| Error::Unavailable)?;
        value["scene"] = director::authored_scene(state).map_err(|_| Error::Unavailable)?;
    }
    wire::validate("StorySnapshot", &value).map_err(|_| Error::Unavailable)?;
    Ok(value)
}

/// Repositories check persisted original-response receipts before calling this.
/// Work on a candidate; caller stores candidate + response atomically or neither.
pub fn execute(state: &State, request: &Value) -> Result<(State, Value), Error> {
    wire::validate("StoryCommand", request).map_err(|_| Error::InvalidInput)?;
    let command: wire::StoryCommand =
        serde_json::from_value(request.clone()).map_err(|_| Error::InvalidInput)?;
    if command.expected_revision != state.revision() {
        return Err(Error::RevisionConflict);
    }
    if state.revision() >= wonderwilds_domain::story::MAX_COMMANDS as u64 {
        return Err(Error::RateLimited);
    }
    let action = match command.payload {
        wire::StoryCommandPayload::Variant1(choice) => {
            director::resolve_action(state, &choice.action_id).map_err(|e| match e {
                director::Error::InvalidActionHandle => Error::RevisionConflict,
                director::Error::CommandBudgetExhausted => Error::RateLimited,
                director::Error::InvalidPayload(_) => Error::Unavailable,
            })?
        }
        wire::StoryCommandPayload::Variant2(ack) => {
            let presented = state.presentation();
            if presented.scene_id != ack.scene_id
                || !acknowledgeable_entries(state).contains(&ack.entry_id.as_str())
            {
                return Err(Error::RevisionConflict);
            }
            Action::Acknowledge {
                scene_id: ack.scene_id,
                entry_id: ack.entry_id,
            }
        }
    };
    let mut candidate = state.clone();
    candidate
        .apply(&Command {
            command_id: command.command_id,
            expected_revision: command.expected_revision,
            action,
        })
        .map_err(|e| match e {
            Rejection::RevisionConflict | Rejection::CommandIdReused | Rejection::NotAvailable => {
                Error::RevisionConflict
            }
            Rejection::LimitReached => Error::RateLimited,
            Rejection::InvalidCheckpoint => Error::Unavailable,
            Rejection::InvalidAction
            | Rejection::InvalidCommand
            | Rejection::InsufficientInventory => Error::InvalidInput,
        })?;
    let response = snapshot(&candidate)?;
    Ok((candidate, response))
}

#[cfg(test)]
mod tests;
