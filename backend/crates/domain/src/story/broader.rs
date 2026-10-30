//! Small authored mechanics shared by discovery, choice, escort and gathering stories.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BroaderProgress {
    pub flags: BTreeSet<String>,
    pub instrument_home: Option<Location>,
    pub corner_style: Option<String>,
    pub bramble_location: Location,
    pub gathering_place: Option<Location>,
}
impl Default for BroaderProgress {
    fn default() -> Self {
        Self {
            flags: BTreeSet::new(),
            instrument_home: None,
            corner_style: None,
            bramble_location: Location::Workshop,
            gathering_place: None,
        }
    }
}

impl State {
    pub fn has_story_flag(&self, id: &str) -> bool {
        self.broader.flags.contains(id)
    }
    pub fn story_flags(&self) -> &BTreeSet<String> {
        &self.broader.flags
    }
    pub fn instrument_home(&self) -> Option<Location> {
        self.broader.instrument_home
    }
    pub fn corner_style(&self) -> Option<&str> {
        self.broader.corner_style.as_deref()
    }
    pub fn bramble_location(&self) -> Location {
        self.broader.bramble_location
    }
    pub fn gathering_place(&self) -> Option<Location> {
        self.broader.gathering_place
    }

    fn flag(&mut self, id: &str, description: &str) -> Result<String, Rejection> {
        if !self.broader.flags.insert(id.into()) {
            return Err(Rejection::NotAvailable);
        }
        Ok(description.into())
    }
    fn need(&self, condition: bool) -> Result<(), Rejection> {
        if condition {
            Ok(())
        } else {
            Err(Rejection::NotAvailable)
        }
    }
    pub(super) fn broader_transition(&mut self, action: &Action) -> Result<String, Rejection> {
        self.require_active()?;
        match action {
            Action::Inspect { object_id } => {
                let (mission, location, prerequisite, flag, description) = match object_id.as_str()
                {
                    "humming_hollow" => (
                        Mission::HummingHollow,
                        Location::FarBank,
                        "hum_heard",
                        "hollow_listened",
                        "The player listened at the hollow and heard the hum among the trees.",
                    ),
                    "trembling_leaf" => (
                        Mission::HummingHollow,
                        Location::FarBank,
                        "hollow_listened",
                        "leaf_clue_found",
                        "A trembling leaf showed the player where the breeze enters the fallen branch.",
                    ),
                    "mossy_stump" => (
                        Mission::HummingHollow,
                        Location::FarBank,
                        "hollow_listened",
                        "mossy_stump_checked",
                        "The player checked the mossy stump; the hum does not come from there.",
                    ),
                    "hollow_branch" => (
                        Mission::HummingHollow,
                        Location::FarBank,
                        "leaf_clue_found",
                        "sound_discovered",
                        "The player discovered that wind passing through the hollow branch makes the humming sound.",
                    ),
                    "corner_site" => (
                        Mission::CosyCorner,
                        Location::BridgeBank,
                        "corner_preferences_known",
                        "corner_site_checked",
                        "The player found a quiet corner beside the stream for Pip.",
                    ),
                    "workshop_path" => (
                        Mission::BramblesVisit,
                        Location::Workshop,
                        "bramble_wish_known",
                        "workshop_path_checked",
                        "The player checked the workshop path and found loose branches to move aside.",
                    ),
                    _ => return Err(Rejection::InvalidAction),
                };
                self.need(
                    self.mission == mission
                        && self.location == location
                        && self.has_story_flag(prerequisite),
                )?;
                self.flag(flag, description)
            }
            Action::Talk {
                character_id,
                topic_id,
            } => {
                let (mission, character, flag, description) = match topic_id.as_str() {
                    "humming_hollow" => (
                        Mission::HummingHollow,
                        "pip",
                        "hum_heard",
                        "Pip told the player about a humming sound on the far bank.",
                    ),
                    "share_discovery" => (
                        Mission::HummingHollow,
                        "pip",
                        "hollow_shared",
                        "The player shared the hollow branch discovery with Pip, keeping the instrument in its chosen place.",
                    ),
                    "cosy_preferences" => (
                        Mission::CosyCorner,
                        "pip",
                        "corner_preferences_known",
                        "Pip said a quiet corner with a view of the stream would feel cosy.",
                    ),
                    "enjoy_corner" => (
                        Mission::CosyCorner,
                        "pip",
                        "corner_shared",
                        "Pip and the player enjoyed the cosy corner in its chosen design.",
                    ),
                    "visit_wish" => (
                        Mission::BramblesVisit,
                        "bramble",
                        "bramble_wish_known",
                        "Bramble said they would like to visit the bridge bank together.",
                    ),
                    "ready_to_visit" => (
                        Mission::BramblesVisit,
                        "bramble",
                        "bramble_ready",
                        "With the path clear, Bramble is ready to walk to the bridge bank with the player.",
                    ),
                    "visit_arrival" => (
                        Mission::BramblesVisit,
                        "bramble",
                        "bramble_visit_shared",
                        "The player welcomed Bramble after their walk together to the bridge bank.",
                    ),
                    "welcome_friends" => (
                        Mission::MoonlightGathering,
                        "pip",
                        "gathering_shared",
                        "Pip, Bramble and the player gathered at the chosen meeting place with seats and a humming circle ready.",
                    ),
                    _ => return Err(Rejection::InvalidAction),
                };
                if character_id != character {
                    return Err(Rejection::InvalidAction);
                }
                let present = if character == "pip" {
                    self.pip_location
                } else {
                    self.bramble_location()
                };
                self.need(self.mission == mission && self.location == present)?;
                self.need(match topic_id.as_str() {
                    "share_discovery" => self.has_story_flag("instrument_placed"),
                    "enjoy_corner" => self.has_story_flag("corner_ready"),
                    "ready_to_visit" => self.has_story_flag("workshop_path_clear"),
                    "visit_arrival" => self.has_story_flag("bramble_arrived"),
                    "welcome_friends" => {
                        self.gathering_place() == Some(self.location)
                            && [
                                "pip_invited",
                                "bramble_invited",
                                "gathering_seats_ready",
                                "gathering_music_ready",
                            ]
                            .iter()
                            .all(|id| self.has_story_flag(id))
                    }
                    _ => true,
                })?;
                self.flag(flag, description)
            }
            Action::Choose {
                choice_id,
                option_id,
            } => {
                match choice_id.as_str() {
                    "instrument_home" => {
                        let home = match option_id.as_str() {
                            "woods" => Location::FarBank,
                            "gathering" => Location::BridgeBank,
                            _ => return Err(Rejection::InvalidAction),
                        };
                        self.need(
                            self.mission == Mission::HummingHollow
                                && self.location == Location::FarBank
                                && self.has_story_flag("sound_discovered")
                                && self.instrument_home().is_none(),
                        )?;
                        self.broader.instrument_home = Some(home);
                        Ok(if home == Location::FarBank { "The player chose to keep the wind instrument in the woods on the far bank." } else { "The player chose to carry the wind instrument to the bridge bank." }.into())
                    }
                    "corner_style" => {
                        if !matches!(option_id.as_str(), "leafy_nook" | "star_canopy") {
                            return Err(Rejection::InvalidAction);
                        }
                        self.need(
                            self.mission == Mission::CosyCorner
                                && self.location == Location::BridgeBank
                                && self.has_story_flag("corner_site_checked")
                                && self.corner_style().is_none(),
                        )?;
                        self.broader.corner_style = Some(option_id.clone());
                        Ok(if option_id == "leafy_nook" {
                            "The player chose a leafy nook for Pip's cosy corner."
                        } else {
                            "The player chose a star canopy for Pip's cosy corner."
                        }
                        .into())
                    }
                    "gathering_place" => {
                        let place = match option_id.as_str() {
                            "near_bank" => Location::BridgeBank,
                            "far_bank" => Location::FarBank,
                            _ => return Err(Rejection::InvalidAction),
                        };
                        self.need(
                            self.mission == Mission::MoonlightGathering
                                && self.location == self.pip_location
                                && self.gathering_place().is_none(),
                        )?;
                        self.broader.gathering_place = Some(place);
                        Ok(format!(
                            "The player chose {} for the moonlight gathering.",
                            place.as_str()
                        ))
                    }
                    _ => Err(Rejection::InvalidAction),
                }
            }
            Action::Invite { character_id } => {
                self.need(self.mission == Mission::MoonlightGathering)?;
                let place = self.gathering_place().ok_or(Rejection::NotAvailable)?;
                match character_id.as_str() {
                    "pip" => {
                        self.need(
                            self.location == self.pip_location
                                && !self.has_story_flag("pip_invited"),
                        )?;
                        self.pip_location = place;
                        self.flag(
                            "pip_invited",
                            &format!(
                                "Pip accepted the invitation and went to the gathering at {}.",
                                place.as_str()
                            ),
                        )
                    }
                    "bramble" => {
                        self.need(
                            self.location == self.bramble_location()
                                && !self.has_story_flag("bramble_invited"),
                        )?;
                        self.broader.bramble_location = place;
                        self.flag(
                            "bramble_invited",
                            &format!(
                                "Bramble accepted the invitation and went to the gathering at {}.",
                                place.as_str()
                            ),
                        )
                    }
                    _ => Err(Rejection::InvalidAction),
                }
            }
            Action::Assemble {
                object_id,
                recipe_id,
            } => {
                let (mission, location, prerequisite, flag, description) = match (
                    object_id.as_str(),
                    recipe_id.as_str(),
                ) {
                    ("wind_instrument", "place_instrument") => (
                        Mission::HummingHollow,
                        self.instrument_home().ok_or(Rejection::NotAvailable)?,
                        Some("sound_discovered"),
                        "instrument_placed",
                        "The player set the wind instrument in its chosen home.",
                    ),
                    ("corner_frame", "build_corner_frame") => (
                        Mission::CosyCorner,
                        Location::BridgeBank,
                        None,
                        "corner_frame_ready",
                        "The player put together the cosy corner frame in the chosen design.",
                    ),
                    ("corner_cushions", "arrange_cushions") => (
                        Mission::CosyCorner,
                        Location::BridgeBank,
                        Some("corner_frame_ready"),
                        "corner_ready",
                        "The player arranged cushions in Pip's cosy corner.",
                    ),
                    ("workshop_path", "clear_path") => (
                        Mission::BramblesVisit,
                        Location::Workshop,
                        Some("workshop_path_checked"),
                        "workshop_path_clear",
                        "The player moved the loose branches aside, clearing the workshop path.",
                    ),
                    ("gathering_seats", "arrange_seats") => (
                        Mission::MoonlightGathering,
                        self.gathering_place().ok_or(Rejection::NotAvailable)?,
                        None,
                        "gathering_seats_ready",
                        "The player arranged seats at the chosen gathering place.",
                    ),
                    ("gathering_music", "prepare_music") => (
                        Mission::MoonlightGathering,
                        self.gathering_place().ok_or(Rejection::NotAvailable)?,
                        None,
                        "gathering_music_ready",
                        "The player prepared a humming circle for friends, leaving the wind instrument in its chosen home.",
                    ),
                    _ => return Err(Rejection::InvalidAction),
                };
                self.need(
                    self.mission == mission
                        && self.location == location
                        && prerequisite.is_none_or(|id| self.has_story_flag(id)),
                )?;
                if mission == Mission::CosyCorner {
                    self.need(self.corner_style().is_some())?;
                }
                self.flag(flag, description)
            }
            Action::Accompany {
                character_id,
                destination_id,
            } => {
                if character_id != "bramble" || destination_id != "bridge_bank" {
                    return Err(Rejection::InvalidAction);
                }
                self.need(
                    self.mission == Mission::BramblesVisit
                        && self.location == Location::Workshop
                        && self.bramble_location() == self.location
                        && self.has_story_flag("bramble_ready"),
                )?;
                self.location = Location::BridgeBank;
                self.broader.bramble_location = Location::BridgeBank;
                self.flag(
                    "bramble_arrived",
                    "The player and Bramble walked together from the workshop to the bridge bank.",
                )
            }
            _ => Err(Rejection::InvalidAction),
        }
    }

    pub(super) fn broader_milestones(&self) -> Vec<(&'static str, &'static str)> {
        match self.mission {
            Mission::HummingHollow => vec![
                ("hum_heard", "Ask Pip about the humming sound."),
                ("hollow_listened", "Listen at the hollow."),
                ("leaf_clue_found", "Look for the trembling leaf."),
                ("sound_discovered", "Discover what makes the sound."),
                (
                    "instrument_home_chosen",
                    "Choose a home for the wind instrument.",
                ),
                (
                    "instrument_placed",
                    "Set the wind instrument in its chosen home.",
                ),
                ("hollow_shared", "Share the discovery with Pip."),
            ],
            Mission::CosyCorner => vec![
                ("corner_preferences_known", "Ask Pip what feels cosy."),
                ("corner_site_checked", "Check the quiet corner."),
                ("corner_style_chosen", "Choose a leafy nook or star canopy."),
                ("corner_frame_ready", "Put together the corner frame."),
                ("corner_ready", "Arrange the cushions."),
                ("corner_shared", "Enjoy the corner with Pip."),
            ],
            Mission::BramblesVisit => vec![
                ("bramble_wish_known", "Ask Bramble about the visit."),
                ("workshop_path_checked", "Check the workshop path."),
                ("workshop_path_clear", "Clear the loose branches."),
                ("bramble_ready", "Check that Bramble is ready."),
                ("bramble_arrived", "Walk to the bridge bank with Bramble."),
                (
                    "bramble_visit_shared",
                    "Welcome Bramble to the bridge bank.",
                ),
            ],
            Mission::MoonlightGathering => vec![
                ("gathering_place_chosen", "Choose the gathering place."),
                ("pip_invited", "Invite Pip."),
                ("bramble_invited", "Invite Bramble."),
                (
                    "gathering_seats_ready",
                    "Arrange seats at the gathering place.",
                ),
                ("gathering_music_ready", "Prepare a humming circle."),
                ("gathering_shared", "Welcome the friends together."),
            ],
            _ => vec![],
        }
    }
    pub(super) fn broader_actions(&self) -> Vec<Action> {
        let mut actions = vec![];
        for object in [
            "humming_hollow",
            "trembling_leaf",
            "mossy_stump",
            "hollow_branch",
            "corner_site",
            "workshop_path",
        ] {
            actions.push(Action::Inspect {
                object_id: object.into(),
            });
        }
        for (character, topic) in [
            ("pip", "humming_hollow"),
            ("pip", "share_discovery"),
            ("pip", "cosy_preferences"),
            ("pip", "enjoy_corner"),
            ("bramble", "visit_wish"),
            ("bramble", "ready_to_visit"),
            ("bramble", "visit_arrival"),
            ("pip", "welcome_friends"),
        ] {
            actions.push(Action::Talk {
                character_id: character.into(),
                topic_id: topic.into(),
            });
        }
        for (choice, options) in [
            ("instrument_home", ["woods", "gathering"]),
            ("corner_style", ["leafy_nook", "star_canopy"]),
            ("gathering_place", ["near_bank", "far_bank"]),
        ] {
            for option in options {
                actions.push(Action::Choose {
                    choice_id: choice.into(),
                    option_id: option.into(),
                });
            }
        }
        for character in ["pip", "bramble"] {
            actions.push(Action::Invite {
                character_id: character.into(),
            });
        }
        for (object, recipe) in [
            ("wind_instrument", "place_instrument"),
            ("corner_frame", "build_corner_frame"),
            ("corner_cushions", "arrange_cushions"),
            ("workshop_path", "clear_path"),
            ("gathering_seats", "arrange_seats"),
            ("gathering_music", "prepare_music"),
        ] {
            actions.push(Action::Assemble {
                object_id: object.into(),
                recipe_id: recipe.into(),
            });
        }
        actions.push(Action::Accompany {
            character_id: "bramble".into(),
            destination_id: "bridge_bank".into(),
        });
        actions
    }
    pub(super) fn broader_presentation(&self) -> (&'static str, &'static str) {
        if self.location == Location::Workshop && !self.met_woodcutter {
            return (
                "woodcutter",
                "Hello! I am the woodcutter. I help neighbours prepare supplies and craftwork.",
            );
        }
        let speaker = if self.location == self.pip_location {
            "pip"
        } else if self.location == self.bramble_location() {
            "bramble"
        } else {
            ""
        };
        if self.mission_status == MissionStatus::Abandoned {
            return (
                speaker,
                "There is time for a break. Your discoveries, choices and preparations will be here when you continue.",
            );
        }
        let text = match (self.mission, self.mission_status) {
            (Mission::HummingHollow, MissionStatus::Completed) => {
                "The humming mystery is solved. The wind instrument stays in the place you chose."
            }
            (Mission::HummingHollow, _) if self.has_story_flag("instrument_placed") => {
                "The wind instrument is in its chosen home. Pip would love to hear what you discovered."
            }
            (Mission::HummingHollow, _) if self.instrument_home().is_some() => {
                "You chose a home for the wind instrument. Set it in place there before sharing the discovery with Pip."
            }
            (Mission::HummingHollow, _) if self.has_story_flag("sound_discovered") => {
                "The breeze makes the hollow branch hum. You can choose where this woodland instrument should stay."
            }
            (Mission::HummingHollow, _) => {
                "Pip heard a hum across the stream. The hollow and its trembling leaf may help you discover the sound."
            }
            (Mission::CosyCorner, MissionStatus::Completed) => {
                "Pip's cosy corner is ready in the design you chose, beside the quiet stream."
            }
            (Mission::CosyCorner, _) if self.has_story_flag("corner_ready") => {
                "The cushions are arranged and the corner is ready. It is time to enjoy it with Pip beside the stream."
            }
            (Mission::CosyCorner, _) if self.has_story_flag("corner_frame_ready") => {
                "The frame is ready in your chosen design. The cushions still need arranging beside the stream."
            }
            (Mission::CosyCorner, _) if self.corner_style().is_some() => {
                "The design is chosen. The frame can now be put together at the quiet corner beside the stream."
            }
            (Mission::CosyCorner, _) if self.has_story_flag("corner_site_checked") => {
                "The quiet spot has a view of the stream. A leafy nook or a star canopy could make it cosy."
            }
            (Mission::CosyCorner, _) if self.has_story_flag("corner_preferences_known") => {
                "Pip would enjoy a quiet view of the stream. There is a corner beside the bridge to check."
            }
            (Mission::CosyCorner, _) => {
                "Pip has an idea for a cosy place. Ask what would make it feel right."
            }
            (Mission::BramblesVisit, MissionStatus::Completed) => {
                "Bramble has arrived at the bridge bank after your walk together."
            }
            (Mission::BramblesVisit, _) if self.has_story_flag("bramble_arrived") => {
                "The walk together is finished. A welcome at the bridge bank will complete the visit."
            }
            (Mission::BramblesVisit, _) if self.has_story_flag("bramble_ready") => {
                "The path is clear and Bramble is ready. You can walk together from the workshop to the bridge bank."
            }
            (Mission::BramblesVisit, _) if self.has_story_flag("workshop_path_clear") => {
                "The loose branches are out of the way. Check with Bramble at the workshop before walking together."
            }
            (Mission::BramblesVisit, _) if self.has_story_flag("workshop_path_checked") => {
                "Loose branches lie on the workshop path. Moving them aside will make room for the visit."
            }
            (Mission::BramblesVisit, _) if self.has_story_flag("bramble_wish_known") => {
                "Bramble would like to visit the bridge bank. The path at the workshop needs checking first."
            }
            (Mission::BramblesVisit, _) => {
                "Bramble is at the workshop with an idea for a visit. Ask where your friend would like to go."
            }
            (Mission::MoonlightGathering, MissionStatus::Completed) => {
                "Pip and Bramble gathered at the place you chose. The seats and humming circle remain ready."
            }
            (Mission::MoonlightGathering, _) if self.gathering_place().is_none() => {
                "Choose which bank should host the gathering. Then friends can be invited and the place prepared."
            }
            (Mission::MoonlightGathering, _)
                if !self.has_story_flag("pip_invited")
                    || !self.has_story_flag("bramble_invited") =>
            {
                "The gathering place is chosen. There are friends still to invite; preparations can happen before or after the invitations."
            }
            (Mission::MoonlightGathering, _)
                if !self.has_story_flag("gathering_seats_ready")
                    && !self.has_story_flag("gathering_music_ready") =>
            {
                "Both friends accepted. Seats and a humming circle still need preparing at the gathering place, in either order."
            }
            (Mission::MoonlightGathering, _) if !self.has_story_flag("gathering_seats_ready") => {
                "Both friends accepted and the humming circle is prepared. The seats still need arranging at the gathering place."
            }
            (Mission::MoonlightGathering, _) if !self.has_story_flag("gathering_music_ready") => {
                "Both friends accepted and the seats are arranged. A humming circle still needs preparing at the gathering place."
            }
            (Mission::MoonlightGathering, _) => {
                "Both friends and all the preparations are ready. Meet at the chosen place and welcome everyone together."
            }
            _ => "There are paths to explore and friends to visit.",
        };
        (speaker, text)
    }
}
