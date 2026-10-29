//! Replay using recorded server facts only.
use super::{Command, Content, Context, Event, Rejection, State, apply};

/// Persist with the event stream. Context contains only recorded server facts;
/// replay never contacts an assessment service, random source, clock or model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayEntry {
    pub command: Command,
    pub context: Context,
    pub events: Vec<Event>,
}
pub fn replay(
    initial: &State,
    content: &Content,
    entries: &[ReplayEntry],
) -> Result<State, Rejection> {
    let mut state = initial.clone();
    for entry in entries {
        let applied = apply(&state, &entry.command, content, &entry.context)?;
        if applied.events != entry.events {
            return Err(Rejection::ReplayMismatch);
        }
        state = applied.state;
    }
    Ok(state)
}
