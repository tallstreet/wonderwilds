//! Shared-device tactics and reciprocal keepsake-copy consent. No learning or XP.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const RULES_VERSION: &str = "paired-1";
pub const MAX_COMPLETED_PRACTICES: u32 = 1_000_000;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    pub expedition_id: String,
    pub navigator_profile_id: String,
    pub keeper_profile_id: String,
    pub companions: BTreeMap<String, String>,
    pub practice: Option<Practice>,
    pub completed_practices: u32,
    pub offers: BTreeMap<String, String>,
    pub confirmed_profile_ids: BTreeSet<String>,
    pub exchange_complete: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Practice {
    pub progress: u32,
    pub pressure: u32,
    pub turns: u32,
    pub next_role: Role,
    pub outcome: Outcome,
    pub history: Vec<Turn>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Turn {
    pub profile_id: String,
    pub companion_id: String,
    pub role: Role,
    pub movement: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Navigator,
    Keeper,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Active,
    Won,
    Retry,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Start {
        #[serde(rename = "partnerProfileId")]
        partner_profile_id: String,
    },
    SwapRoles,
    SelectCompanion {
        #[serde(rename = "companionId")]
        companion_id: String,
    },
    StartPractice,
    PracticeTurn {
        movement: String,
    },
    OfferKeepsake {
        #[serde(rename = "keepsakeId")]
        keepsake_id: String,
    },
    ConfirmExchange,
    ContinueSolo,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Forbidden,
    Conflict,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyGrant {
    pub recipient: String,
    pub source: String,
    pub keepsake_id: String,
}
#[derive(Clone, Debug)]
pub struct Applied {
    pub state: Option<State>,
    pub copies: Vec<CopyGrant>,
}
pub fn whitelisted_keepsake(id: &str) -> bool {
    matches!(
        id,
        "copy_of_watchtower_clue"
            | "duplicate_rubbing"
            | "garland_ridge_clue"
            | "leaf_fossil_keepsake"
            | "shell_fossil_keepsake"
            | "leaf_fossil_rubbing"
            | "shell_fossil_rubbing"
            | "island_charm"
            | "rowan_first_ribbon"
            | "rowan_shared_ribbon"
            | "rowan_final_ribbon"
    ) || [
        "pip", "tansy", "nib", "flutter", "cleo", "bramble", "patter", "twizzle", "luna", "pebble",
        "sail", "glimmer",
    ]
    .iter()
    .any(|friend| id == format!("{friend}_keepsake"))
}
impl State {
    pub fn role(&self, actor: &str) -> Option<Role> {
        if actor == self.navigator_profile_id {
            Some(Role::Navigator)
        } else if actor == self.keeper_profile_id {
            Some(Role::Keeper)
        } else {
            None
        }
    }
    pub fn participants(&self) -> [&str; 2] {
        [&self.navigator_profile_id, &self.keeper_profile_id]
    }
    fn swap(&mut self) {
        std::mem::swap(&mut self.navigator_profile_id, &mut self.keeper_profile_id);
    }
}
pub fn validate(s: &State) -> Result<(), Error> {
    let ids = s.participants();
    let valid_id = |id: &str| !id.is_empty() && id.len() <= 120;
    if !valid_id(&s.expedition_id)
        || !valid_id(ids[0])
        || !valid_id(ids[1])
        || ids[0] == ids[1]
        || s.companions.len() > 2
        || s.offers.len() > 2
        || s.confirmed_profile_ids.len() > 2
        || s.completed_practices > MAX_COMPLETED_PRACTICES
        || s.companions
            .iter()
            .any(|(p, c)| !ids.contains(&p.as_str()) || !valid_id(c))
        || s.offers
            .iter()
            .any(|(p, k)| !ids.contains(&p.as_str()) || !whitelisted_keepsake(k))
        || s.confirmed_profile_ids
            .iter()
            .any(|p| !s.offers.contains_key(p))
        || (s.exchange_complete && (s.offers.len() != 2 || s.confirmed_profile_ids.len() != 2))
    {
        return Err(Error::Invalid);
    }
    if let Some(p) = &s.practice {
        if p.progress > 7
            || p.pressure > 5
            || p.turns > 8
            || p.history.len() != p.turns as usize
            || p.history.len() > 8
        {
            return Err(Error::Invalid);
        }
        let original_nav = if p.outcome == Outcome::Active {
            &s.navigator_profile_id
        } else {
            &s.keeper_profile_id
        };
        let original_keeper = if p.outcome == Outcome::Active {
            &s.keeper_profile_id
        } else {
            &s.navigator_profile_id
        };
        let (mut progress, mut pressure, mut outcome) = (0u32, 0u32, Outcome::Active);
        for (index, t) in p.history.iter().enumerate() {
            let role = if index % 2 == 0 {
                Role::Navigator
            } else {
                Role::Keeper
            };
            let actor = if role == Role::Navigator {
                original_nav
            } else {
                original_keeper
            };
            if outcome != Outcome::Active
                || t.role != role
                || &t.profile_id != actor
                || s.companions.get(actor) != Some(&t.companion_id)
            {
                return Err(Error::Invalid);
            }
            match (role, t.movement.as_str()) {
                (Role::Navigator, "scout") | (Role::Keeper, "rally") => {
                    progress += 2;
                    pressure += 1
                }
                (Role::Navigator, "rest") => pressure = pressure.saturating_sub(2),
                (Role::Keeper, "brace") => {
                    progress += 1;
                    pressure = pressure.saturating_sub(1)
                }
                _ => return Err(Error::Invalid),
            }
            if progress >= 6 {
                outcome = Outcome::Won
            } else if pressure >= 5 || index + 1 >= 8 {
                outcome = Outcome::Retry
            }
        }
        if p.progress != progress
            || p.pressure != pressure
            || p.outcome != outcome
            || p.next_role
                != if p.turns % 2 == 0 {
                    Role::Navigator
                } else {
                    Role::Keeper
                }
            || (outcome == Outcome::Won && s.completed_practices == 0)
        {
            return Err(Error::Invalid);
        }
    }
    Ok(())
}
/// Ownership arguments come from the verified actor's canonical profile. Both
/// offered originals are independently rechecked by persistence before copying.
pub fn apply(
    current: Option<&State>,
    actor: &str,
    command_id: &str,
    command: &Command,
    owned_companions: &BTreeSet<String>,
    owned_keepsakes: &BTreeSet<String>,
) -> Result<Applied, Error> {
    if let Command::Start { partner_profile_id } = command {
        if current.is_some() {
            return Err(Error::Conflict);
        }
        if actor == partner_profile_id || actor.is_empty() || partner_profile_id.is_empty() {
            return Err(Error::Invalid);
        }
        return Ok(Applied {
            state: Some(State {
                expedition_id: command_id.into(),
                navigator_profile_id: actor.into(),
                keeper_profile_id: partner_profile_id.clone(),
                ..State::default()
            }),
            copies: vec![],
        });
    }
    let mut s = current.cloned().ok_or(Error::Conflict)?;
    validate(&s)?;
    let role = s.role(actor).ok_or(Error::Forbidden)?;
    let mut copies = vec![];
    match command {
        Command::Start { .. } => unreachable!(),
        Command::ContinueSolo => {
            return Ok(Applied {
                state: None,
                copies,
            });
        }
        Command::SwapRoles => {
            if s.practice
                .as_ref()
                .is_some_and(|p| p.outcome == Outcome::Active)
            {
                return Err(Error::Conflict);
            }
            s.practice = None;
            s.swap();
        }
        Command::SelectCompanion { companion_id } => {
            if !owned_companions.contains(companion_id) {
                return Err(Error::Forbidden);
            }
            if s.practice
                .as_ref()
                .is_some_and(|p| p.outcome == Outcome::Active)
            {
                return Err(Error::Conflict);
            }
            s.practice = None;
            s.companions.insert(actor.into(), companion_id.clone());
        }
        Command::StartPractice => {
            if role != Role::Navigator {
                return Err(Error::Forbidden);
            }
            if s.companions.len() != 2
                || s.practice
                    .as_ref()
                    .is_some_and(|p| p.outcome == Outcome::Active)
            {
                return Err(Error::Conflict);
            }
            s.practice = Some(Practice {
                progress: 0,
                pressure: 0,
                turns: 0,
                next_role: Role::Navigator,
                outcome: Outcome::Active,
                history: vec![],
            });
        }
        Command::PracticeTurn { movement } => {
            let friend = s.companions.get(actor).ok_or(Error::Conflict)?.clone();
            if !owned_companions.contains(&friend) {
                return Err(Error::Forbidden);
            }
            let p = s.practice.as_mut().ok_or(Error::Conflict)?;
            if p.outcome != Outcome::Active || p.next_role != role {
                return Err(Error::Forbidden);
            }
            match (role, movement.as_str()) {
                (Role::Navigator, "scout") | (Role::Keeper, "rally") => {
                    p.progress += 2;
                    p.pressure += 1
                }
                (Role::Navigator, "rest") => p.pressure = p.pressure.saturating_sub(2),
                (Role::Keeper, "brace") => {
                    p.progress += 1;
                    p.pressure = p.pressure.saturating_sub(1)
                }
                _ => return Err(Error::Invalid),
            }
            p.turns += 1;
            p.history.push(Turn {
                profile_id: actor.into(),
                companion_id: friend,
                role,
                movement: movement.clone(),
            });
            p.next_role = if role == Role::Navigator {
                Role::Keeper
            } else {
                Role::Navigator
            };
            if p.progress >= 6 {
                p.outcome = Outcome::Won;
                s.completed_practices = s
                    .completed_practices
                    .checked_add(1)
                    .filter(|count| *count <= MAX_COMPLETED_PRACTICES)
                    .ok_or(Error::Conflict)?;
                s.swap();
            } else if p.pressure >= 5 || p.turns >= 8 {
                p.outcome = Outcome::Retry;
                s.swap();
            }
        }
        Command::OfferKeepsake { keepsake_id } => {
            if s.exchange_complete {
                return Err(Error::Conflict);
            }
            if !whitelisted_keepsake(keepsake_id) || !owned_keepsakes.contains(keepsake_id) {
                return Err(Error::Forbidden);
            }
            s.offers.insert(actor.into(), keepsake_id.clone());
            s.confirmed_profile_ids.clear();
        }
        Command::ConfirmExchange => {
            if s.exchange_complete || s.offers.len() != 2 {
                return Err(Error::Conflict);
            }
            let original = s.offers.get(actor).ok_or(Error::Conflict)?;
            if !owned_keepsakes.contains(original) {
                return Err(Error::Forbidden);
            }
            s.confirmed_profile_ids.insert(actor.into());
            if s.confirmed_profile_ids.len() == 2 {
                copies.push(CopyGrant {
                    recipient: s.navigator_profile_id.clone(),
                    source: s.keeper_profile_id.clone(),
                    keepsake_id: s.offers[&s.keeper_profile_id].clone(),
                });
                copies.push(CopyGrant {
                    recipient: s.keeper_profile_id.clone(),
                    source: s.navigator_profile_id.clone(),
                    keepsake_id: s.offers[&s.navigator_profile_id].clone(),
                });
                s.exchange_complete = true;
            }
        }
    }
    Ok(Applied {
        state: Some(s),
        copies,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|x| (*x).into()).collect()
    }
    #[test]
    fn roles_owned_turns_and_copy_consent_are_actor_bound() {
        let mut state = apply(
            None,
            "a",
            "pair",
            &Command::Start {
                partner_profile_id: "b".into(),
            },
            &set(&[]),
            &set(&[]),
        )
        .unwrap()
        .state;
        for (a, c) in [("a", "pip"), ("b", "tansy")] {
            state = apply(
                state.as_ref(),
                a,
                "select",
                &Command::SelectCompanion {
                    companion_id: c.into(),
                },
                &set(&[c]),
                &set(&[]),
            )
            .unwrap()
            .state;
        }
        assert!(
            apply(
                state.as_ref(),
                "b",
                "start",
                &Command::StartPractice,
                &set(&["tansy"]),
                &set(&[])
            )
            .is_err()
        );
        state = apply(
            state.as_ref(),
            "a",
            "start",
            &Command::StartPractice,
            &set(&["pip"]),
            &set(&[]),
        )
        .unwrap()
        .state;
        assert!(
            apply(
                state.as_ref(),
                "b",
                "wrong",
                &Command::PracticeTurn {
                    movement: "scout".into()
                },
                &set(&["tansy"]),
                &set(&[])
            )
            .is_err()
        );
        for a in ["a", "b", "a", "b"] {
            let movement = if a == "a" { "scout" } else { "brace" };
            state = apply(
                state.as_ref(),
                a,
                "turn",
                &Command::PracticeTurn {
                    movement: movement.into(),
                },
                &set(&[if a == "a" { "pip" } else { "tansy" }]),
                &set(&[]),
            )
            .unwrap()
            .state;
        }
        assert_eq!(state.as_ref().unwrap().navigator_profile_id, "b");
        assert_eq!(state.as_ref().unwrap().completed_practices, 1);
        for (a, k) in [("a", "pip_keepsake"), ("b", "tansy_keepsake")] {
            state = apply(
                state.as_ref(),
                a,
                "offer",
                &Command::OfferKeepsake {
                    keepsake_id: k.into(),
                },
                &set(&[]),
                &set(&[k]),
            )
            .unwrap()
            .state;
        }
        let first = apply(
            state.as_ref(),
            "a",
            "confirm",
            &Command::ConfirmExchange,
            &set(&[]),
            &set(&["pip_keepsake"]),
        )
        .unwrap();
        assert!(first.copies.is_empty());
        let second = apply(
            first.state.as_ref(),
            "b",
            "confirm2",
            &Command::ConfirmExchange,
            &set(&[]),
            &set(&["tansy_keepsake"]),
        )
        .unwrap();
        assert_eq!(second.copies.len(), 2);
        assert!(
            apply(
                second.state.as_ref(),
                "a",
                "again",
                &Command::ConfirmExchange,
                &set(&[]),
                &set(&["pip_keepsake"])
            )
            .is_err()
        );
        assert!(
            apply(
                second.state.as_ref(),
                "a",
                "solo",
                &Command::ContinueSolo,
                &set(&[]),
                &set(&[])
            )
            .unwrap()
            .state
            .is_none()
        );
    }
    #[test]
    fn completion_cap_accepts_last_win_and_rejects_next_without_mutating_state() {
        let nearly_won = |completed_practices| State {
            expedition_id: "bounded-practice".into(),
            navigator_profile_id: "a".into(),
            keeper_profile_id: "b".into(),
            companions: BTreeMap::from([("a".into(), "pip".into()), ("b".into(), "tansy".into())]),
            practice: Some(Practice {
                progress: 5,
                pressure: 1,
                turns: 3,
                next_role: Role::Keeper,
                outcome: Outcome::Active,
                history: vec![
                    Turn {
                        profile_id: "a".into(),
                        companion_id: "pip".into(),
                        role: Role::Navigator,
                        movement: "scout".into(),
                    },
                    Turn {
                        profile_id: "b".into(),
                        companion_id: "tansy".into(),
                        role: Role::Keeper,
                        movement: "brace".into(),
                    },
                    Turn {
                        profile_id: "a".into(),
                        companion_id: "pip".into(),
                        role: Role::Navigator,
                        movement: "scout".into(),
                    },
                ],
            }),
            completed_practices,
            ..State::default()
        };
        let last = nearly_won(MAX_COMPLETED_PRACTICES - 1);
        validate(&last).unwrap();
        let winning_turn = Command::PracticeTurn {
            movement: "brace".into(),
        };
        let accepted = apply(
            Some(&last),
            "b",
            "last-win",
            &winning_turn,
            &set(&["tansy"]),
            &set(&[]),
        )
        .unwrap()
        .state
        .unwrap();
        assert_eq!(accepted.completed_practices, MAX_COMPLETED_PRACTICES);
        validate(&accepted).unwrap();
        let capped = nearly_won(MAX_COMPLETED_PRACTICES);
        let original = capped.clone();
        validate(&capped).unwrap();
        assert_eq!(
            apply(
                Some(&capped),
                "b",
                "over-cap",
                &winning_turn,
                &set(&["tansy"]),
                &set(&[])
            )
            .unwrap_err(),
            Error::Conflict
        );
        assert_eq!(capped, original);
        assert!(
            apply(
                Some(&capped),
                "b",
                "solo-at-cap",
                &Command::ContinueSolo,
                &set(&[]),
                &set(&[])
            )
            .unwrap()
            .state
            .is_none()
        );
    }
}
