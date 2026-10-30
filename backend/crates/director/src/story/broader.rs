//! Authored descriptions of the discovery, choice, escort and event mechanics.
use super::*;

pub(super) fn is_broader(mission: Mission) -> bool {
    matches!(
        mission,
        Mission::HummingHollow
            | Mission::CosyCorner
            | Mission::BramblesVisit
            | Mission::MoonlightGathering
    )
}
pub(super) fn visible(state: &State) -> bool {
    is_broader(state.mission())
}

pub(super) fn conditions(mission: Mission) -> Vec<(&'static str, &'static str)> {
    match mission {
        Mission::HummingHollow => vec![
            ("hum_heard", "Ask Pip about the hum."),
            ("hollow_listened", "Listen at the hollow."),
            ("leaf_clue_found", "Inspect the trembling leaf."),
            ("sound_discovered", "Find the sound in the hollow branch."),
            (
                "instrument_home_chosen",
                "Choose a lasting home for the instrument.",
            ),
            (
                "instrument_placed",
                "Place the instrument in its chosen home.",
            ),
            ("hollow_shared", "Share the discovery with Pip."),
        ],
        Mission::CosyCorner => vec![
            ("corner_preferences_known", "Listen to Pip's preferences."),
            ("corner_site_checked", "Inspect the quiet corner."),
            (
                "corner_style_chosen",
                "Choose the leafy nook or star canopy.",
            ),
            ("corner_frame_ready", "Build the chosen frame."),
            ("corner_ready", "Arrange the cushions."),
            ("corner_shared", "Enjoy the corner with Pip."),
        ],
        Mission::BramblesVisit => vec![
            ("bramble_wish_known", "Ask Bramble about a visit."),
            ("workshop_path_checked", "Check the workshop path."),
            ("workshop_path_clear", "Clear loose branches from the path."),
            ("bramble_ready", "Ask whether Bramble is ready."),
            ("bramble_arrived", "Walk with Bramble to the bridge bank."),
            ("bramble_visit_shared", "Welcome Bramble after arriving."),
        ],
        Mission::MoonlightGathering => vec![
            ("gathering_place_chosen", "Choose a bank for the gathering."),
            ("pip_invited", "Invite Pip to the chosen place."),
            ("bramble_invited", "Invite Bramble to the chosen place."),
            (
                "gathering_seats_ready",
                "Arrange seats at the chosen place.",
            ),
            (
                "gathering_music_ready",
                "Prepare a humming circle at the chosen place.",
            ),
            (
                "gathering_shared",
                "Welcome both friends when preparations are ready.",
            ),
        ],
        _ => vec![],
    }
}

pub(super) fn facts(state: &State) -> Vec<String> {
    let mut facts = Vec::new();
    if state.mission() == Mission::WelcomeLights {
        facts.push("Lanterns may be placed on either bank first.".into());
    }
    if !visible(state) {
        return facts;
    }
    facts.push(format!(
        "Bramble is at {}.",
        state.bramble_location().as_str()
    ));
    for (flag, fact) in [
        (
            "hum_heard",
            "Pip told the player about a hum on the far bank.",
        ),
        (
            "hollow_listened",
            "The player listened at the humming hollow.",
        ),
        (
            "leaf_clue_found",
            "A trembling leaf led the player to the hollow branch.",
        ),
        (
            "mossy_stump_checked",
            "The mossy stump is not the source of the hum.",
        ),
        (
            "sound_discovered",
            "Wind passing through the hollow branch makes the humming sound.",
        ),
        ("hollow_shared", "The player shared the discovery with Pip."),
        (
            "corner_preferences_known",
            "Pip likes a quiet place to watch the stream.",
        ),
        ("corner_frame_ready", "The cosy corner frame is ready."),
        (
            "corner_shared",
            "Pip and the player enjoyed the cosy corner.",
        ),
        ("workshop_path_clear", "The workshop path is clear."),
        ("bramble_ready", "Bramble is ready for the visit."),
        (
            "bramble_arrived",
            "The player and Bramble walked to the bridge bank together.",
        ),
        ("pip_invited", "Pip has joined the chosen gathering place."),
        (
            "bramble_invited",
            "Bramble has joined the chosen gathering place.",
        ),
        ("gathering_seats_ready", "Gathering seats are ready."),
        ("gathering_music_ready", "Gathering music is ready."),
        ("gathering_shared", "The moonlight gathering has begun."),
    ] {
        let clue_only = matches!(
            flag,
            "hum_heard" | "hollow_listened" | "leaf_clue_found" | "mossy_stump_checked"
        );
        if clue_only && state.mission() != Mission::HummingHollow {
            continue;
        }
        if flag == "corner_frame_ready" && state.has_story_flag("corner_ready") {
            continue;
        }
        if state.has_story_flag(flag) {
            facts.push(fact.into());
        }
    }
    if let Some(home) = state.instrument_home() {
        facts.push(format!(
            "The chosen wind instrument home is {}.",
            home.as_str()
        ));
        if state.has_story_flag("instrument_placed") {
            facts.push(format!("The wind instrument is at {}.", home.as_str()));
        }
    }
    if let Some(style) = state.corner_style() {
        facts.push(format!("The chosen cosy corner design is {style}."));
        if state.has_story_flag("corner_ready") {
            facts.push(format!("Pip's cosy corner uses {style}."));
        }
    }
    if let Some(place) = state.gathering_place() {
        facts.push(format!("The gathering is at {}.", place.as_str()));
    }
    if state.mission() == Mission::MoonlightGathering {
        facts.push("Friends may be invited and seats and music prepared in any order after choosing the place.".into());
        facts.push("The gathering music is a humming circle; the wind instrument stays in its chosen home.".into());
    }
    facts
}

pub(super) fn objects(state: &State) -> Vec<Value> {
    if !visible(state) {
        return vec![];
    }
    let mut objects = vec![];
    let mut add = |id: &str, value: &str| objects.push(json!({"id":id,"state":value}));
    match state.location() {
        Location::FarBank if state.mission() == Mission::HummingHollow => {
            for (id, flag) in [
                ("humming_hollow", "hollow_listened"),
                ("trembling_leaf", "leaf_clue_found"),
                ("mossy_stump", "mossy_stump_checked"),
                ("hollow_branch", "sound_discovered"),
            ] {
                if id == "hollow_branch" && state.instrument_home().is_some() {
                    continue;
                }
                add(
                    id,
                    if state.has_story_flag(flag) {
                        "discovered"
                    } else {
                        "not yet investigated"
                    },
                );
            }
        }
        Location::BridgeBank if state.mission() == Mission::CosyCorner => {
            add(
                "corner_site",
                if state.has_story_flag("corner_site_checked") {
                    "checked"
                } else {
                    "quiet place"
                },
            );
            add(
                "corner_frame",
                if state.has_story_flag("corner_frame_ready") {
                    "ready"
                } else {
                    "not built"
                },
            );
            add(
                "corner_cushions",
                if state.has_story_flag("corner_ready") {
                    "arranged"
                } else {
                    "not arranged"
                },
            );
        }
        Location::Workshop => add(
            "workshop_path",
            if state.has_story_flag("workshop_path_clear") {
                "clear"
            } else {
                "loose branches"
            },
        ),
        _ => {}
    }
    if state.instrument_home() == Some(state.location()) {
        add(
            "wind_instrument",
            if state.has_story_flag("instrument_placed") {
                "in its chosen home"
            } else {
                "ready to place"
            },
        );
    }
    if state.has_story_flag("corner_ready") && state.location() == Location::BridgeBank {
        add("cosy_corner", state.corner_style().unwrap());
    }
    if state.gathering_place() == Some(state.location()) {
        add(
            "gathering_seats",
            if state.has_story_flag("gathering_seats_ready") {
                "ready"
            } else {
                "not arranged"
            },
        );
        add(
            "gathering_music",
            if state.has_story_flag("gathering_music_ready") {
                "humming circle ready"
            } else {
                "not prepared"
            },
        );
    }
    objects
}

pub(super) fn assemble_description(object: &str, _recipe: &str) -> &'static str {
    match object {
        "wind_instrument" => "Place the wind instrument in its chosen home.",
        "corner_frame" => "Build the frame for Pip's chosen corner.",
        "corner_cushions" => "Arrange the cushions in Pip's corner.",
        "workshop_path" => "Move the loose branches off the path.",
        "gathering_seats" => "Arrange seats for the gathering.",
        "gathering_music" => "Prepare a humming circle for the gathering.",
        _ => "Prepare this place.",
    }
}
pub(super) fn action_description(action: &Action) -> String {
    match action {
        Action::Inspect { object_id } => match object_id.as_str() {
            "humming_hollow" => "Listen for the hum in the hollow.",
            "trembling_leaf" => "Look closely at the trembling leaf.",
            "mossy_stump" => "Check whether the hum comes from the mossy stump.",
            "hollow_branch" => "Inspect the hollow branch.",
            "corner_site" => "Look at the quiet corner beside the stream.",
            "workshop_path" => "Check the path from the workshop.",
            _ => "Look closely.",
        }
        .into(),
        Action::Talk { topic_id, .. } => match topic_id.as_str() {
            "humming_hollow" => "Ask Pip about the humming sound.",
            "share_discovery" => "Tell Pip what makes the humming sound.",
            "cosy_preferences" => "Ask what Pip would like in a cosy corner.",
            "enjoy_corner" => "Enjoy the finished corner with Pip.",
            "visit_wish" => "Ask Bramble about visiting the bridge bank.",
            "ready_to_visit" => "Ask whether Bramble is ready to walk.",
            "visit_arrival" => "Welcome Bramble to the bridge bank.",
            "welcome_friends" => "Welcome everyone to the moonlight gathering.",
            _ => "Talk together.",
        }
        .into(),
        Action::Choose { option_id, .. } => match option_id.as_str() {
            "woods" => "Choose the woods as the instrument home.",
            "gathering" => "Pick up the wind instrument to take to the bridge bank.",
            "leafy_nook" => "Choose a leafy nook for Pip.",
            "star_canopy" => "Choose a star canopy for Pip.",
            "near_bank" => "Hold the gathering on the near bank.",
            "far_bank" => "Hold the gathering on the far bank.",
            _ => "Choose this idea.",
        }
        .into(),
        Action::Invite { character_id } => format!(
            "Invite {} to the gathering.",
            if character_id == "pip" {
                "Pip"
            } else {
                "Bramble"
            }
        ),
        _ => "Continue the story.".into(),
    }
}

pub(super) fn capabilities(state: &State, mission: Mission) -> Vec<Value> {
    if !is_broader(mission) {
        return vec![];
    }
    let mut result = vec![];
    let mut add = |action: Value, at: &str, needs: Vec<&str>, effects: Vec<&str>| {
        result.push(json!({"action":action,"at":at,"requires":needs,"effects":effects}))
    };
    match mission {
        Mission::HummingHollow => {
            add(
                json!({"type":"talk","characterId":"pip","topicId":"humming_hollow"}),
                "bridge_bank",
                vec!["Active humming mission", "Pip present"],
                vec!["Learn about the humming sound"],
            );
            for (object, need, effect) in [
                (
                    "humming_hollow",
                    "Pip told the player about the hum",
                    "Listen at the hollow",
                ),
                ("trembling_leaf", "Hollow listened to", "Find the leaf clue"),
                (
                    "mossy_stump",
                    "Hollow listened to",
                    "Rule out the mossy stump",
                ),
                (
                    "hollow_branch",
                    "Leaf clue found",
                    "Discover the wind makes the sound",
                ),
            ] {
                add(
                    json!({"type":"inspect","objectId":object}),
                    "far_bank",
                    vec!["Active humming mission", need],
                    vec![effect],
                );
            }
            for option in ["woods", "gathering"] {
                add(
                    json!({"type":"choose","choiceId":"instrument_home","optionId":option}),
                    "far_bank",
                    vec![
                        "Active humming mission",
                        "Sound discovered",
                        "Home not chosen",
                    ],
                    vec!["Choose the instrument home; carry it when moving to the near bank"],
                );
            }
            for location in ["bridge_bank", "far_bank"] {
                add(
                    json!({"type":"assemble","objectId":"wind_instrument","recipeId":"place_instrument"}),
                    location,
                    vec![
                        "Active humming mission",
                        "This is the chosen instrument home",
                        "Instrument not yet placed",
                    ],
                    vec!["Place the instrument in its chosen home"],
                );
            }
            add(
                json!({"type":"talk","characterId":"pip","topicId":"share_discovery"}),
                "bridge_bank",
                vec!["Active humming mission", "Instrument placed", "Pip present"],
                vec!["Share the discovery and complete the mission"],
            );
        }
        Mission::CosyCorner => {
            add(
                json!({"type":"talk","characterId":"pip","topicId":"cosy_preferences"}),
                "bridge_bank",
                vec!["Active corner mission", "Pip present"],
                vec!["Learn Pip's preferences"],
            );
            add(
                json!({"type":"inspect","objectId":"corner_site"}),
                "bridge_bank",
                vec!["Active corner mission", "Preferences known"],
                vec!["Check the corner site"],
            );
            for option in ["leafy_nook", "star_canopy"] {
                add(
                    json!({"type":"choose","choiceId":"corner_style","optionId":option}),
                    "bridge_bank",
                    vec!["Active corner mission", "Site checked", "Design not chosen"],
                    vec!["Remember the chosen design"],
                );
            }
            for (object, recipe, need, effect) in [
                (
                    "corner_frame",
                    "build_corner_frame",
                    "Design chosen",
                    "Build the chosen frame",
                ),
                (
                    "corner_cushions",
                    "arrange_cushions",
                    "Frame ready",
                    "Finish the cosy corner",
                ),
            ] {
                add(
                    json!({"type":"assemble","objectId":object,"recipeId":recipe}),
                    "bridge_bank",
                    vec!["Active corner mission", need],
                    vec![effect],
                );
            }
            add(
                json!({"type":"talk","characterId":"pip","topicId":"enjoy_corner"}),
                "bridge_bank",
                vec!["Active corner mission", "Corner ready", "Pip present"],
                vec!["Enjoy the corner and complete the mission"],
            );
        }
        Mission::BramblesVisit => {
            for (topic, need, effect) in [
                (
                    "visit_wish",
                    "Bramble present",
                    "Learn Bramble's destination",
                ),
                (
                    "ready_to_visit",
                    "Path clear and Bramble present",
                    "Bramble is ready to walk",
                ),
            ] {
                add(
                    json!({"type":"talk","characterId":"bramble","topicId":topic}),
                    "workshop",
                    vec!["Active visit mission", need],
                    vec![effect],
                );
            }
            add(
                json!({"type":"inspect","objectId":"workshop_path"}),
                "workshop",
                vec!["Active visit mission", "Visit wish known"],
                vec!["Find loose branches on the path"],
            );
            add(
                json!({"type":"assemble","objectId":"workshop_path","recipeId":"clear_path"}),
                "workshop",
                vec!["Active visit mission", "Path checked"],
                vec!["Clear the path"],
            );
            add(
                json!({"type":"accompany","characterId":"bramble","destinationId":"bridge_bank"}),
                "workshop",
                vec![
                    "Active visit mission",
                    "Bramble ready and present",
                    "Path clear",
                ],
                vec!["Player and Bramble arrive at the bridge bank"],
            );
            add(
                json!({"type":"talk","characterId":"bramble","topicId":"visit_arrival"}),
                "bridge_bank",
                vec![
                    "Active visit mission",
                    "Shared walk recorded",
                    "Bramble present",
                ],
                vec!["Welcome Bramble and complete the visit"],
            );
        }
        Mission::MoonlightGathering => {
            for option in ["near_bank", "far_bank"] {
                add(
                    json!({"type":"choose","choiceId":"gathering_place","optionId":option}),
                    state.pip_location().as_str(),
                    vec![
                        "Active gathering mission",
                        "Pip present",
                        "Place not chosen",
                    ],
                    vec!["Remember the gathering place"],
                );
            }
            for (character, location) in [
                ("pip", state.pip_location()),
                ("bramble", state.bramble_location()),
            ] {
                add(
                    json!({"type":"invite","characterId":character}),
                    location.as_str(),
                    vec![
                        "Active gathering mission",
                        "Place chosen",
                        "Friend present and not yet invited",
                    ],
                    vec!["Friend joins the chosen gathering place"],
                );
            }
            for location in [Location::BridgeBank, Location::FarBank] {
                for (object, recipe, effect) in [
                    ("gathering_seats", "arrange_seats", "Seats ready"),
                    (
                        "gathering_music",
                        "prepare_music",
                        "Humming circle ready; instrument home unchanged",
                    ),
                ] {
                    add(
                        json!({"type":"assemble","objectId":object,"recipeId":recipe}),
                        location.as_str(),
                        vec![
                            "Active gathering mission",
                            "This is the chosen gathering place",
                        ],
                        vec![effect],
                    );
                }
                add(
                    json!({"type":"talk","characterId":"pip","topicId":"welcome_friends"}),
                    location.as_str(),
                    vec![
                        "Active gathering mission",
                        "This is the chosen gathering place",
                        "Both friends invited",
                        "Seats and music ready",
                    ],
                    vec!["Gathering begins and mission completes"],
                );
            }
        }
        _ => {}
    }
    result
}

pub(super) fn scene(input: &Value, canonical: &Value) -> Result<Value, Error> {
    let status = input["mission"]["status"].as_str().unwrap();
    let mission = input["mission"]["missionId"].as_str().unwrap();
    let actions = input["availableActions"].as_array().unwrap();
    let progress = |a: &&Value| {
        !matches!(
            a["action"]["type"].as_str(),
            Some("travel" | "cancel_mission" | "accept_mission")
        )
    };
    let primary = actions
        .iter()
        .filter(progress)
        .find(|a| a["action"]["objectId"] != "mossy_stump");
    let line = if status == "abandoned" {
        "We can take a break. Your discoveries and choices will be here when you choose to continue.".to_owned()
    } else if status == "completed" {
        match mission {
            "humming_hollow"=>"We found what makes the hum! The wind instrument will stay in the home you chose.",
            "cosy_corner"=>"Pip's cosy corner is ready. The design you chose is now part of our place beside the stream.",
            "brambles_visit"=>"Bramble has arrived after your walk together. It is lovely to welcome a friend to the bridge bank.",
            _=>"The friends have gathered at the place you chose. The seats and humming circle are ready!",
        }.into()
    } else if let Some(action) = primary {
        match action["action"]["type"].as_str().unwrap() {
            "talk" => match action["action"]["topicId"].as_str().unwrap() {
                "humming_hollow" => "A soft hum is coming from the far bank. Pip has something to tell you about it.",
                "share_discovery" => "You found the wind's hollow branch and chose its home. Pip is ready to hear your discovery.",
                "cosy_preferences" => "A cosy corner should feel right for Pip. Start by finding out what your friend would enjoy.",
                "enjoy_corner" => "Come and sit with Pip beside the stream.",
                "visit_wish" => "Bramble would like to visit the bridge bank. Have a chat before setting out together.",
                "ready_to_visit" => "The path is clear. Check whether Bramble is ready before you start your walk together.",
                "visit_arrival" => "You and Bramble have arrived together. Welcome your friend to this side of the workshop path.",
                _ => "The seats and humming circle are ready, and both friends have joined you. Welcome everyone to the gathering.",
            }.to_owned(),
            "inspect" => match action["action"]["objectId"].as_str().unwrap() {
                "humming_hollow" => "The hollow is nearby. Listen closely to begin following the sound.",
                "trembling_leaf" => "The hum is among the trees. A trembling leaf may help you find where the breeze goes.",
                "mossy_stump" => "You can check the mossy stump as you follow the sound. The leaf and branch are nearby too.",
                "hollow_branch" => "The leaf points towards a hollow branch. Look closely to discover how it makes its sound.",
                "corner_site" => "Pip likes watching the stream. Take a look at the quiet corner beside it.",
                _ => "Look along the workshop path before taking Bramble on the visit.",
            }.to_owned(),
            "choose" => match action["action"]["choiceId"].as_str().unwrap() {
                "instrument_home" => "The breeze makes the branch hum! Choose to leave it in the woods or place it near the gathering space.",
                "corner_style" => "A leafy nook or a star canopy can make Pip's corner cosy. Choose the design you would like to build.",
                _ => "Choose a bank for the gathering. Then you can invite friends and prepare the place in any order.",
            }.to_owned(),
            "accompany" => "Bramble is ready and the path is clear. Choose to walk to the bridge bank together.".to_owned(),
            _ => action["description"].as_str().unwrap().to_owned(),
        }
    } else {
        travel_guidance(input, mission)
    };
    let mut entries = vec![];
    let required_intro = if super::mock::needs_introduction(input) {
        canonical["presentation"].as_array().unwrap().first()
    } else {
        None
    };
    if let Some(intro) = required_intro {
        entries.push(intro.clone());
    }
    let present = input["here"]["characters"].as_array().unwrap();
    if let Some((speaker, text)) = confirmed_result(input["context"]["trigger"].as_str().unwrap()) {
        if !speaker.is_empty() && present.iter().any(|c| c["id"] == speaker) {
            entries.push(json!({"type":"dialogue","entryId":format!("{mission}-result"),"speakerId":speaker,"text":text}));
        } else {
            entries.push(
                json!({"type":"narration","entryId":format!("{mission}-result"),"text":text}),
            );
        }
    }
    entries.push(json!({"type":"narration","entryId":format!("{mission}-next"),"text":line}));
    for c in present {
        let id = c["id"].as_str().unwrap();
        if entries.iter().any(|e| e["speakerId"] == id) {
            continue;
        }
        let voice = match id {
            "pip" if status == "abandoned" => {
                "Take your time. I'll be glad to help when you choose to continue."
            }
            "pip" if mission == "humming_hollow" && status == "completed" => {
                "A woodland branch making music! We can listen for it when we visit its home."
            }
            "pip"
                if mission == "humming_hollow"
                    && has_fact(
                        input,
                        "Wind passing through the hollow branch makes the humming sound.",
                    ) =>
            {
                "I'm ready to hear what you found by the hollow branch."
            }
            "pip" if mission == "humming_hollow" => "We can follow the clues one at a time.",
            "pip" if mission == "cosy_corner" && status == "completed" => {
                "This is a lovely place to watch the stream. Thank you for making it with me!"
            }
            "pip"
                if mission == "cosy_corner"
                    && has_fact(input, "Pip's cosy corner uses leafy_nook.") =>
            {
                "The leafy nook feels peaceful. I like the gentle shade of its leaves."
            }
            "pip"
                if mission == "cosy_corner"
                    && has_fact(input, "Pip's cosy corner uses star_canopy.") =>
            {
                "I like the star canopy above us. It makes this spot feel special."
            }
            "pip"
                if mission == "cosy_corner"
                    && has_fact(input, "The cosy corner frame is ready.") =>
            {
                "This spot has such a lovely view of the stream."
            }
            "pip" if mission == "cosy_corner" => {
                "I would love a corner that feels cosy. We can make it together."
            }
            "pip" => "It is lovely to spend time with friends beside the stream.",
            "bramble" if status == "abandoned" => {
                "A little break is fine. We can carry on when you choose."
            }
            "bramble"
                if actions.iter().any(|a| {
                    a["action"]["type"] == "accompany" && a["action"]["characterId"] == "bramble"
                }) =>
            {
                "I'm ready now. Shall we walk to the bridge bank together?"
            }
            "bramble" if mission == "brambles_visit" && status == "completed" => {
                "I'm glad we made the visit together. Thank you for showing me the way!"
            }
            "bramble" => "We can take our time and enjoy being together.",
            _ => "You are welcome to stop by while you explore with your friends.",
        };
        entries.push(json!({"type":"dialogue","entryId":format!("{mission}-{id}"),"speakerId":id,"text":voice}));
    }
    let mut result = canonical.clone();
    result["presentation"] = json!(entries);
    Ok(result)
}

fn has_fact(input: &Value, fact: &str) -> bool {
    input["context"]["memory"]["facts"]
        .as_array()
        .is_some_and(|facts| facts.iter().any(|value| value == fact))
}

/// Name the reachable route towards the next milestone instead of asking the
/// player to guess between several paths. Only use currently offered travel.
fn travel_guidance(input: &Value, mission: &str) -> String {
    let destination = match mission {
        "brambles_visit" => "workshop",
        "cosy_corner" => "bridge_bank",
        "humming_hollow"
            if has_fact(input, "The wind instrument is at far_bank.")
                || has_fact(input, "The chosen wind instrument home is bridge_bank.") =>
        {
            "bridge_bank"
        }
        "humming_hollow" => "far_bank",
        "moonlight_gathering" if has_fact(input, "The gathering is at far_bank.") => "far_bank",
        _ => "bridge_bank",
    };
    let actions = input["availableActions"].as_array().unwrap();
    let travel = actions
        .iter()
        .find(|a| a["action"]["type"] == "travel" && a["action"]["destinationId"] == destination)
        .or_else(|| {
            actions.iter().find(|a| {
                a["action"]["type"] == "travel" && a["action"]["destinationId"] == "bridge_bank"
            })
        });
    let current = input["mission"]["current"]
        .as_str()
        .unwrap_or("Explore together.");
    match travel {
        Some(action) => format!(
            "{} Next: {current}",
            action["description"].as_str().unwrap()
        ),
        None => format!("Next: {current}"),
    }
}

/// Explicit authored outcomes from the committed runtime event. Never expose the
/// developer-facing event string as child prose or treat a proposed choice as done.
fn confirmed_result(trigger: &str) -> Option<(&'static str, &'static str)> {
    let rules = [
        (
            "Pip told the player about a humming sound",
            "pip",
            "I heard a gentle hum on the far bank. Could you listen near the trees and see what makes it?",
        ),
        (
            "The player listened at the hollow",
            "",
            "A gentle humming sound drifts among the trees. Look nearby for a clue.",
        ),
        (
            "A trembling leaf showed the player",
            "",
            "The leaf trembles where the breeze enters a hollow branch. Follow that clue to the sound.",
        ),
        (
            "The player checked the mossy stump",
            "",
            "The mossy stump is quiet. The humming sound must be coming from somewhere else.",
        ),
        (
            "The player discovered that wind passing",
            "",
            "The wind passes through the hollow branch and makes it hum. You found a woodland wind instrument!",
        ),
        (
            "The player shared the hollow branch discovery",
            "pip",
            "A branch making music with the breeze! I'm glad you found its secret and chose a home for it.",
        ),
        (
            "Pip said a quiet corner",
            "pip",
            "A quiet place where I can watch the stream would feel cosy. A leafy nook or a star canopy would be lovely.",
        ),
        (
            "The player found a quiet corner",
            "",
            "This quiet corner has a view of the stream. It is a good spot for Pip's cosy place.",
        ),
        (
            "Pip and the player enjoyed the cosy corner",
            "pip",
            "The cushions and the design you chose make this a lovely place to watch the stream. Thank you!",
        ),
        (
            "Bramble said they would like",
            "bramble",
            "I'd like to visit the bridge bank with you. Could we check the path before we set out?",
        ),
        (
            "The player checked the workshop path",
            "",
            "There are loose branches along the path. Moving them aside will make Bramble's walk easier.",
        ),
        (
            "The player moved the loose branches aside",
            "",
            "The loose branches are out of the way. The workshop path is clear for your walk with Bramble.",
        ),
        (
            "With the path clear, Bramble is ready",
            "bramble",
            "The path looks clear. I'm ready to walk to the bridge bank with you now!",
        ),
        (
            "The player welcomed Bramble",
            "bramble",
            "Thank you for walking with me. I'm happy to be here by the bridge!",
        ),
        (
            "The player chose to keep the wind instrument",
            "",
            "You chose the woods as the instrument's home. Find a spot here to set it in place.",
        ),
        (
            "The player chose to carry the wind instrument",
            "",
            "You picked up the wind instrument to bring it to the bridge bank. Cross back to place it in its new home.",
        ),
        (
            "The player set the wind instrument",
            "",
            "The wind instrument is now in the home you chose. It will stay there as your story continues.",
        ),
        (
            "The player chose a leafy nook",
            "",
            "A leafy nook it is! You can build its frame here beside the stream.",
        ),
        (
            "The player chose a star canopy",
            "",
            "A star canopy it is! You can build its frame here beside the stream.",
        ),
        (
            "The player put together the cosy corner frame",
            "",
            "The frame matches your chosen design. Arrange the cushions to finish Pip's cosy corner.",
        ),
        (
            "The player arranged cushions",
            "",
            "The cushions are arranged in Pip's cosy corner.",
        ),
        (
            "The player chose bridge_bank for the moonlight",
            "",
            "The gathering will be on the near bank. Invite your friends and prepare the seats and humming circle in any order.",
        ),
        (
            "The player chose far_bank for the moonlight",
            "",
            "The gathering will be on the far bank. Invite your friends and prepare the seats and humming circle in any order.",
        ),
        (
            "Pip accepted the invitation",
            "",
            "Pip accepted your invitation and went to the gathering place you chose.",
        ),
        (
            "Bramble accepted the invitation",
            "",
            "Bramble accepted your invitation and went to the gathering place you chose.",
        ),
        (
            "The player arranged seats at the chosen",
            "",
            "The seats are arranged at your chosen gathering place. There will be room for your friends.",
        ),
        (
            "The player prepared a humming circle",
            "",
            "The humming circle is ready for your friends. The wind instrument stays in the home you chose earlier.",
        ),
        (
            "Pip, Bramble and the player gathered",
            "pip",
            "The seats and music are ready, and we are all here together. Welcome to our moonlight gathering!",
        ),
    ];
    rules
        .into_iter()
        .find_map(|(prefix, speaker, text)| trigger.starts_with(prefix).then_some((speaker, text)))
}
