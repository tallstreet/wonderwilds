//! WW-05 headless gameplay and replay, independent of HTTP, clocks and models.
//!
//! `TrustedFacts` is deliberately not deserializable. The integration layer must
//! construct it from server-issued encounters and persisted assessment, in the
//! same transaction as state/evidence/reward writes. Never build it from a JSON
//! request, guest import, director output or a client correctness assertion.

pub const RULES_VERSION: &str = "domain-1.0.0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    InvalidContent(String),
    InvalidCommand,
    RevisionConflict,
    CommandIdReused,
    Locked,
    NotFound,
    AlreadyCompleted,
    UnverifiedEncounter,
    InsufficientInventory,
    Overflow,
    ReplayMismatch,
}

mod content;
mod replay;
#[cfg(test)]
mod tests;
mod transitions;

pub use content::{Content, TrainingDefinition};
pub use replay::{ReplayEntry, replay};
pub use transitions::apply;

mod state;
pub use state::{
    Action, Applied, Command, Context, Event, Outcome, State, TrustedFacts, VerifiedCompletion,
};

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:./".contains(&b))
}

/// Execute only after the caller's durable receipt lookup. Original responses and
/// command identity must be committed with this state in the same transaction.
/// The standalone `apply` API retains its receipt semantics unchanged.
pub fn apply_checkpointed(
    state: &State,
    command: &Command,
    content: &Content,
    context: &Context,
) -> Result<Applied, Rejection> {
    let mut applied = apply(&state.compact(), command, content, context)?;
    applied.state = applied.state.compact();
    Ok(applied)
}
