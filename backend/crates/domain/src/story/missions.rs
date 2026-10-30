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
    #[serde(rename = "humming_hollow")]
    HummingHollow,
    #[serde(rename = "cosy_corner")]
    CosyCorner,
    #[serde(rename = "brambles_visit")]
    BramblesVisit,
    #[serde(rename = "moonlight_gathering")]
    MoonlightGathering,
}
impl Mission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bridge => "pips_bridge",
            Self::Picnic => "pips_picnic",
            Self::WelcomeLights => "welcome_lights",
            Self::HummingHollow => "humming_hollow",
            Self::CosyCorner => "cosy_corner",
            Self::BramblesVisit => "brambles_visit",
            Self::MoonlightGathering => "moonlight_gathering",
        }
    }
    pub fn goal(self) -> &'static str {
        match self {
            Self::Bridge => "Help Pip cross the stream.",
            Self::Picnic => "Prepare a picnic and share it with Pip.",
            Self::WelcomeLights => "Light both banks to welcome friends.",
            Self::HummingHollow => "Discover the humming sound and share it with Pip.",
            Self::CosyCorner => "Make a cosy corner that Pip will enjoy.",
            Self::BramblesVisit => "Help Bramble visit the bridge bank.",
            Self::MoonlightGathering => "Bring friends together for a moonlight gathering.",
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
            Self::HummingHollow => {
                "Pip heard a hum across the stream. Follow its clues and decide where the woodland instrument should stay."
            }
            Self::CosyCorner => {
                "Pip would like a quiet place to watch the stream. Listen to Pip, choose a design and make the corner together."
            }
            Self::BramblesVisit => {
                "Bramble would like to visit the bridge bank. Check the workshop path, clear it and walk there together."
            }
            Self::MoonlightGathering => {
                "Choose a place for friends to meet. Invite Pip and Bramble, arrange seats and prepare a humming circle in any order."
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
            Self::HummingHollow => {
                "Share the discovery with Pip; the wind instrument remains where you chose."
            }
            Self::CosyCorner => {
                "Enjoy the finished leafy nook or star canopy with Pip beside the stream."
            }
            Self::BramblesVisit => {
                "Welcome Bramble to the bridge bank after walking there together."
            }
            Self::MoonlightGathering => {
                "See Pip and Bramble gathered at your chosen place, with seats and a humming circle ready."
            }
        }
    }
    pub fn participant_ids(self) -> &'static [&'static str] {
        match self {
            Self::Bridge | Self::Picnic | Self::WelcomeLights => &["pip", "woodcutter"],
            Self::HummingHollow | Self::CosyCorner => &["pip"],
            Self::BramblesVisit | Self::MoonlightGathering => &["pip", "bramble"],
        }
    }
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Bridge => Some(Self::Picnic),
            Self::Picnic => Some(Self::WelcomeLights),
            Self::WelcomeLights => Some(Self::HummingHollow),
            Self::HummingHollow => Some(Self::CosyCorner),
            Self::CosyCorner => Some(Self::BramblesVisit),
            Self::BramblesVisit => Some(Self::MoonlightGathering),
            Self::MoonlightGathering => None,
        }
    }
}
