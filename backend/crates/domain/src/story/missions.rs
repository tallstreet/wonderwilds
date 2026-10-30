use serde::{Deserialize, Serialize};

/// Ordered authored opportunities. Completion is recorded before a new mission is accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mission {
    #[serde(rename = "pips_bridge")]
    Bridge,
    #[serde(rename = "pips_picnic")]
    Picnic,
    #[serde(rename = "welcome_lights")]
    WelcomeLights,
}
impl Mission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bridge => "pips_bridge",
            Self::Picnic => "pips_picnic",
            Self::WelcomeLights => "welcome_lights",
        }
    }
    pub fn goal(self) -> &'static str {
        match self {
            Self::Bridge => "Help Pip cross the stream.",
            Self::Picnic => "Prepare a picnic and share it with Pip.",
            Self::WelcomeLights => "Light both banks to welcome friends.",
        }
    }
    pub fn premise(self) -> &'static str {
        match self {
            Self::Bridge => {
                "Pip wants to explore the far bank with the player. Help make a way across together."
            }
            Self::Picnic => {
                "After crossing together, Pip would like to share a picnic. Gather berries and ask the woodcutter to pack them."
            }
            Self::WelcomeLights => {
                "After the picnic, Pip wants both banks to feel welcoming. Gather glow pebbles and make lanterns for the path."
            }
        }
    }
    pub fn payoff(self) -> &'static str {
        match self {
            Self::Bridge => {
                "Cross the repaired bridge with Pip and celebrate reaching the far bank."
            }
            Self::Picnic => "Bring Pip back to the prepared picnic and enjoy it together.",
            Self::WelcomeLights => {
                "See lanterns glowing on both banks, with the bridge and picnic preserved."
            }
        }
    }
    pub fn participant_ids(self) -> &'static [&'static str] {
        &["pip", "woodcutter"]
    }
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Bridge => Some(Self::Picnic),
            Self::Picnic => Some(Self::WelcomeLights),
            Self::WelcomeLights => None,
        }
    }
}
