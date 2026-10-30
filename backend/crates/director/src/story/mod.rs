//! Deterministic authored story projections; no provider, identity or learning data.
//!
//! See README.md for scope. Runtime state is the only evidence source. The small
//! catalogue below describes the bound actions implemented by domain::story.
use serde_json::{Value, json};
use wonderwilds_contracts::validate;
use wonderwilds_domain::story::{Action, Location, Mission, MissionStatus, State};

mod broader;
pub mod evaluation;
mod mock;
pub mod planning_evaluation;
pub use mock::{MockSceneProvider, SceneProvider, scene_with_provider, validate_scene};

pub const PROTOCOL: &str = "story-proposal-1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Plan,
    Scene,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPayload(String),
    InvalidActionHandle,
    CommandBudgetExhausted,
}

fn ensure_budget(state: &State) -> Result<(), Error> {
    if state.revision() >= wonderwilds_domain::story::MAX_COMMANDS as u64 {
        Err(Error::CommandBudgetExhausted)
    } else {
        Ok(())
    }
}

fn checked(contract: &str, value: Value) -> Result<Value, Error> {
    if serde_json::to_vec(&value)
        .map_err(|e| Error::InvalidPayload(e.to_string()))?
        .len()
        > 24 * 1024
    {
        return Err(Error::InvalidPayload("Story payload exceeds 24 KiB".into()));
    }
    validate(contract, &value).map_err(Error::InvalidPayload)?;
    Ok(value)
}

fn description(location: Location) -> &'static str {
    match location {
        Location::BridgeBank => "A bridge beside a quiet stream, with a path to the workshop.",
        Location::Workshop => "An open-air workshop beside the stream path.",
        Location::FarBank => "A quiet clearing across the stream.",
    }
}
fn narrative(mission: Mission) -> Value {
    json!({"premise":mission.premise(),"participantIds":mission.participant_ids(),"payoff":mission.payoff()})
}
fn character(state: &State, id: &str) -> Value {
    match id {
        "pip" => {
            json!({"id":"pip","description":"A curious and patient friend.","relationship":if state.picnic_shared(){"The player and Pip crossed the bridge and shared a picnic together."}else if state.crossed_with_pip(){"The player and Pip crossed the bridge together."}else if !state.milestones().is_empty(){"The player accepted a mission to cross the stream with Pip."}else{"No prior shared mission is recorded."}})
        }
        "bramble" => {
            json!({"id":"bramble","description":"A thoughtful friend who enjoys visiting new places at a gentle pace.","relationship":if state.has_story_flag("bramble_arrived"){"The player helped Bramble visit the bridge bank."}else{"Bramble is getting to know the player."}})
        }
        _ => {
            json!({"id":"woodcutter","description":"A patient neighbour who shapes wood.","relationship":if state.met_woodcutter() {"The game has presented the woodcutter's introduction."} else {"No recorded presentation of the woodcutter's introduction."}})
        }
    }
}
fn context(state: &State) -> Value {
    let inventory: Vec<_> = state
        .inventory()
        .into_iter()
        .map(|(item, quantity)| json!({"itemId":item,"quantity":quantity}))
        .collect();
    let mut facts = vec![
        if state.bridge_repaired() {
            "The bridge is repaired."
        } else {
            "The bridge is broken."
        }
        .to_owned(),
    ];
    facts.push(format!("Pip is at {}.", state.pip_location().as_str()));
    if state.met_woodcutter() {
        facts.push("The game has presented the woodcutter's introduction.".into());
    }
    if state.crossed_with_pip() {
        facts.push("The player and Pip crossed the bridge together.".into());
    }
    for (true_now, fact) in [
        (state.picnic_ready(), "The picnic table is ready."),
        (state.picnic_shared(), "The player and Pip shared a picnic."),
        (state.near_lit(), "Two lanterns light the near bank."),
        (state.far_lit(), "One lantern lights the far bank."),
    ] {
        if true_now {
            facts.push(fact.into());
        }
    }
    facts.extend(broader::facts(state));
    let events = state.events();
    let recent: Vec<_> = events
        .iter()
        .skip(events.len().saturating_sub(6))
        .map(|e| e.description.clone())
        .collect();
    let trigger = events
        .last()
        .map_or("The authored bridge story is ready.", |e| {
            e.description.as_str()
        });
    let recent_missions: Vec<_> = state
        .completed_missions()
        .iter()
        .map(|m| format!("{} Completed.", m.goal()))
        .collect();
    json!({"trigger":trigger,"player":{"language":"short_simple","interests":[],"avoid":[]},"locationId":state.location().as_str(),"inventory":inventory,"memory":{"facts":facts,"recentEvents":recent,"recentMissions":recent_missions}})
}

fn mission_conditions(mission: Mission) -> Vec<(&'static str, &'static str)> {
    match mission {
        Mission::Bridge => vec![
            (
                "wood_ready",
                "Have at least three compatible wood; existing wood counts.",
            ),
            (
                "planks_ready",
                "Have at least three compatible planks; existing planks count.",
            ),
            (
                "planks_at_bridge",
                "Be at the bridge bank carrying at least three planks.",
            ),
            ("bridge_repaired", "The bridge is repaired."),
            (
                "crossed_with_pip",
                "The player and Pip have completed a shared crossing.",
            ),
        ],
        Mission::Picnic => vec![
            ("berries_ready", "Collect three berries."),
            ("picnic_basket_ready", "Have the packed picnic basket."),
            ("picnic_prepared", "Set the picnic table."),
            ("picnic_shared", "Bring Pip back and share the picnic."),
        ],
        Mission::WelcomeLights => vec![
            ("glow_pebbles_ready", "Collect three glow pebbles."),
            ("lanterns_ready", "Have three lanterns ready."),
            ("near_bank_lit", "Place two lanterns on the near bank."),
            ("far_bank_lit", "Place one lantern on the far bank."),
        ],
        other => broader::conditions(other),
    }
}

fn milestone_definitions(state: &State) -> Vec<(&str, &str)> {
    if !state.milestones().is_empty() {
        return state
            .milestones()
            .iter()
            .map(|m| (m.condition_id.as_str(), m.label.as_str()))
            .collect();
    }
    let mut result = Vec::new();
    if !state.bridge_repaired() {
        if state.planks() < 3 {
            result.push(("wood_ready", "Find suitable wood."));
        }
        result.push(("planks_ready", "Get the planks ready."));
        result.push(("planks_at_bridge", "Bring the planks to the bridge."));
    }
    result.push(("bridge_repaired", "Repair the bridge."));
    result.push(("crossed_with_pip", "Cross with Pip."));
    result
}

pub fn authored_plan(state: &State) -> Result<Value, Error> {
    checked(
        "StoryMissionPlan",
        json!({"opportunityId":state.mission_id(),"goal":state.mission().goal(),"narrative":narrative(state.mission()),"milestones":milestone_definitions(state).into_iter().map(|(id,label)|json!({"label":label,"conditionId":id})).collect::<Vec<_>>()}),
    )
}

/// Planning is a projection of this small authored opportunity, not acceptance of
/// arbitrary generated missions. No provider is called.
pub fn build_plan_input(state: &State) -> Result<Value, Error> {
    ensure_budget(state)?;
    let mut characters = Vec::new();
    for (id, loc) in [
        ("pip", state.pip_location()),
        ("woodcutter", Location::Workshop),
    ] {
        let mut c = character(state, id);
        c["locationId"] = json!(loc.as_str());
        characters.push(c);
    }
    if broader::visible(state) {
        let mut c = character(state, "bramble");
        c["locationId"] = json!(state.bramble_location().as_str());
        characters.push(c);
    }
    let opportunity = if state.mission_status() == MissionStatus::Completed {
        state.next_mission()
    } else {
        Some(state.mission())
    };
    let opportunities:Vec<_> = opportunity.into_iter().map(|mission| json!({"id":mission.as_str(),"description":mission.premise(),"outcomeId":mission.as_str(),"outcome":mission.payoff(),"conditions":mission_conditions(mission).into_iter().map(|(id,meaning)|json!({"id":id,"meaning":meaning})).collect::<Vec<_>>()})).collect();
    let gate = if state.bridge_repaired() {
        Value::Null
    } else {
        json!("Bridge repaired")
    };
    checked(
        "StoryPlanInput",
        json!({"mode":"plan_mission","context":context(state),"area":{
        "locations":([Location::BridgeBank,Location::Workshop,Location::FarBank].iter().map(|loc|json!({"id":loc.as_str(),"description":description(*loc)})).collect::<Vec<_>>()),
        "routes":[{"from":"bridge_bank","to":"workshop","requirement":null},{"from":"workshop","to":"bridge_bank","requirement":null},{"from":"bridge_bank","to":"far_bank","requirement":gate},{"from":"far_bank","to":"bridge_bank","requirement":gate}],
        "characters":characters},"opportunities":opportunities,"capabilities":capabilities(state,opportunity.unwrap_or(state.mission()))}),
    )
}

fn capabilities(state: &State, mission: Mission) -> Vec<Value> {
    let mut result = Vec::new();
    for (from, to) in [
        ("bridge_bank", "workshop"),
        ("workshop", "bridge_bank"),
        ("bridge_bank", "far_bank"),
        ("far_bank", "bridge_bank"),
    ] {
        let needs: Vec<&str> = if from == "far_bank" || to == "far_bank" {
            vec!["Bridge repaired"]
        } else {
            vec![]
        };
        result.push(json!({"action":{"type":"travel","destinationId":to},"at":from,"requires":needs,"effects":[format!("Player arrives at {to}; inventory unchanged.")]}));
    }
    if mission == Mission::Bridge && !state.bridge_repaired() {
        for (source, at) in [("fallen_wood", "bridge_bank"), ("spare_wood", "workshop")] {
            let count = state.source_remaining(source).unwrap_or(0);
            if count > 0 {
                result.push(json!({"action":{"type":"collect","sourceId":source,"itemId":"wood","quantity":count},"at":at,"requires":["Active mission",format!("Source has {count} wood")],"effects":[format!("Source loses {count} wood"),format!("Player gains {count} wood")]}));
            }
        }
        result.push(json!({"action":{"type":"exchange","characterId":"woodcutter","recipeId":"wood_to_planks"},"at":"workshop","requires":["Active mission","Player has 3 wood","Bridge broken"],"effects":["Consume 3 wood","Grant 3 planks"]}));
        result.push(json!({"action":{"type":"assemble","objectId":"bridge","recipeId":"repair_bridge"},"at":"bridge_bank","requires":["Active mission","Player has 3 planks","Bridge broken"],"effects":["Consume 3 planks","Bridge becomes repaired"]}));
    }
    if mission == Mission::Bridge && !state.crossed_with_pip() {
        result.push(json!({"action":{"type":"accompany","characterId":"pip","destinationId":"far_bank"},"at":"bridge_bank","requires":["Active mission","Bridge repaired","Pip present"],"effects":["Player and Pip arrive at far bank","Record shared crossing"]}));
    }
    if mission == Mission::Picnic {
        let remaining = state.source_remaining("berry_bush").unwrap_or(0);
        if remaining > 0 {
            result.push(json!({"action":{"type":"collect","sourceId":"berry_bush","itemId":"berries","quantity":remaining},"at":"far_bank","requires":["Active picnic mission",format!("Source has {remaining} berries")],"effects":[format!("Consume {remaining} source berries"),format!("Gain {remaining} berries")]}));
        }
        if !state.picnic_ready() {
            result.push(json!({"action":{"type":"exchange","characterId":"woodcutter","recipeId":"berries_to_picnic"},"at":"workshop","requires":["Active picnic mission","Have 3 berries"],"effects":["Consume 3 berries","Gain 1 picnic basket"]}));
            result.push(json!({"action":{"type":"assemble","objectId":"picnic_table","recipeId":"prepare_picnic"},"at":"bridge_bank","requires":["Active picnic mission","Have 1 picnic basket"],"effects":["Consume 1 picnic basket","Picnic table ready"]}));
        }
        if !state.picnic_shared() {
            result.push(json!({"action":{"type":"accompany","characterId":"pip","destinationId":"bridge_bank"},"at":"far_bank","requires":["Active picnic mission","Picnic table ready","Pip present"],"effects":["Player and Pip arrive at bridge bank","Picnic shared"]}));
        }
    }
    if mission == Mission::WelcomeLights {
        let remaining = state.source_remaining("glow_stones").unwrap_or(0);
        if remaining > 0 {
            result.push(json!({"action":{"type":"collect","sourceId":"glow_stones","itemId":"glow_pebbles","quantity":remaining},"at":"far_bank","requires":["Active lights mission",format!("Source has {remaining} glow pebbles")],"effects":[format!("Consume {remaining} source pebbles"),format!("Gain {remaining} glow pebbles")]}));
        }
        if !state.near_lit() && !state.far_lit() {
            result.push(json!({"action":{"type":"exchange","characterId":"woodcutter","recipeId":"glow_to_lanterns"},"at":"workshop","requires":["Active lights mission","Have 3 glow pebbles"],"effects":["Consume 3 glow pebbles","Gain 3 lanterns"]}));
        }
        for (lit, object, recipe, at, count) in [
            (
                state.near_lit(),
                "near_lanterns",
                "light_near_bank",
                "bridge_bank",
                2,
            ),
            (
                state.far_lit(),
                "far_lantern",
                "light_far_bank",
                "far_bank",
                1,
            ),
        ] {
            if !lit {
                result.push(json!({"action":{"type":"assemble","objectId":object,"recipeId":recipe},"at":at,"requires":["Active lights mission",format!("Have {count} lanterns")],"effects":[format!("Consume {count} lanterns"),format!("{object} becomes lit")]}));
            }
        }
    }
    result.extend(broader::capabilities(state, mission));
    result
}

fn action_description(state: &State, action: &Action) -> String {
    match action {
        Action::AcceptMission if state.mission_status() == MissionStatus::Completed => state
            .next_mission()
            .map_or("Explore with Pip.".into(), |m| m.goal().into()),
        Action::AcceptMission if state.mission_status() == MissionStatus::Abandoned => {
            match state.mission() {
                Mission::Bridge => "Keep helping Pip with the bridge.",
                Mission::Picnic => "Keep preparing Pip's picnic.",
                Mission::WelcomeLights => "Keep lighting the banks.",
                _ => return format!("Continue: {}", state.mission().goal()),
            }
            .into()
        }
        Action::AcceptMission => state.mission().goal().into(),
        Action::CancelMission => "Take a break from this mission.".into(),
        Action::Travel { destination_id } => {
            let destination = match destination_id.as_str() {
                "bridge_bank" => "the bridge bank",
                "workshop" => "the workshop",
                "far_bank" => "the far bank",
                _ => "the next clearing",
            };
            format!("Walk to {destination}.")
        }
        Action::Collect {
            source_id,
            quantity,
            ..
        } => {
            if source_id == "berry_bush" {
                return format!("Pick {quantity} berries for the picnic.");
            }
            if source_id == "glow_stones" {
                return format!("Collect {quantity} glow pebbles for lanterns.");
            }
            if source_id == "fallen_wood" {
                format!("Collect {quantity} pieces of wood from fallen branches.")
            } else {
                format!("Collect {quantity} pieces of spare wood.")
            }
        }
        Action::Exchange { recipe_id, .. } => match recipe_id.as_str() {
            "berries_to_picnic" => "Ask the woodcutter to pack the berries into a picnic basket.",
            "glow_to_lanterns" => "Ask the woodcutter to make three glowing lanterns.",
            _ => "Ask the woodcutter to turn 3 pieces of wood into 3 planks.",
        }
        .into(),
        Action::Assemble {
            object_id,
            recipe_id,
        } if broader::is_broader(state.mission()) => {
            broader::assemble_description(object_id, recipe_id).into()
        }
        Action::Assemble { object_id, .. } => match object_id.as_str() {
            "picnic_table" => "Set out the picnic basket on the table.",
            "near_lanterns" => "Place two lanterns on the near bank.",
            "far_lantern" => "Place one lantern on the far bank.",
            _ => "Use 3 planks to repair the bridge.",
        }
        .into(),
        Action::Accompany { character_id, .. } if character_id == "bramble" => {
            "Walk with Bramble to the bridge bank.".into()
        }
        Action::Accompany { destination_id, .. } if destination_id == "bridge_bank" => {
            "Bring Pip back across the bridge to share the picnic.".into()
        }
        Action::Accompany { .. } => "Cross the repaired bridge together with Pip.".into(),
        Action::Acknowledge { .. } => "Continue.".into(),
        other => broader::action_description(other),
    }
}
fn action_handle(state: &State, index: usize) -> String {
    format!("story-v3-{}-{index}", state.revision())
}

/// Exact equality with the current eligible list rejects stale or invented handles.
/// The caller must still execute with expected_revision and a stable command ID.
pub fn resolve_action(state: &State, handle: &str) -> Result<Action, Error> {
    ensure_budget(state)?;
    state
        .available_actions()
        .into_iter()
        .enumerate()
        .find_map(|(i, a)| (action_handle(state, i) == handle).then_some(a))
        .ok_or(Error::InvalidActionHandle)
}

pub fn build_scene_input(state: &State) -> Result<Value, Error> {
    ensure_budget(state)?;
    let mut characters = Vec::new();
    if state.pip_location() == state.location() {
        characters.push(character(state, "pip"));
    }
    if state.location() == Location::Workshop {
        characters.push(character(state, "woodcutter"));
    }
    if broader::visible(state) && state.bramble_location() == state.location() {
        characters.push(character(state, "bramble"));
    }
    let mut objects = match state.location() {
        Location::BridgeBank => vec![
            json!({"id":"bridge","state":if state.bridge_repaired(){"repaired"}else{"broken"}}),
            json!({"id":"fallen_wood","state":format!("{} wood available",state.source_remaining("fallen_wood").unwrap_or(0))}),
        ],
        Location::Workshop => vec![
            json!({"id":"workbench","state":"ready"}),
            json!({"id":"spare_wood","state":format!("{} wood available",state.source_remaining("spare_wood").unwrap_or(0))}),
        ],
        Location::FarBank => vec![],
    };
    match state.location() {
        Location::BridgeBank => {
            objects.push(
                json!({"id":"picnic_table","state":if state.picnic_ready(){"ready"}else{"empty"}}),
            );
            objects.push(
                json!({"id":"near_lanterns","state":if state.near_lit(){"lit"}else{"unlit"}}),
            );
        }
        Location::FarBank => {
            objects.push(json!({"id":"berry_bush","state":format!("{} berries available",state.source_remaining("berry_bush").unwrap_or(0))}));
            objects.push(json!({"id":"glow_stones","state":format!("{} glow pebbles available",state.source_remaining("glow_stones").unwrap_or(0))}));
            objects
                .push(json!({"id":"far_lantern","state":if state.far_lit(){"lit"}else{"unlit"}}));
        }
        Location::Workshop => {}
    }
    objects.extend(broader::objects(state));
    let mission = if matches!(
        state.mission_status(),
        MissionStatus::Active | MissionStatus::Abandoned | MissionStatus::Completed
    ) {
        let completed: Vec<_> = state
            .milestones()
            .iter()
            .filter(|m| m.complete)
            .map(|m| m.label.clone())
            .collect();
        let pending: Vec<_> = state
            .milestones()
            .iter()
            .filter(|m| !m.complete)
            .map(|m| m.label.clone())
            .collect();
        json!({"missionId":state.mission_id(),"goal":state.mission().goal(),"status":state.mission_status().as_str(),"narrative":narrative(state.mission()),"completed":completed,"current":pending.first(),"remaining":pending.iter().skip(1).collect::<Vec<_>>()})
    } else {
        Value::Null
    };
    let actions: Vec<_> = state.available_actions().iter().enumerate().map(|(i,a)|json!({"id":action_handle(state,i),"description":action_description(state,a),"action":a})).collect();
    checked(
        "StorySceneInput",
        json!({"mode":"continue_story","context":context(state),"here":{"description":description(state.location()),"characters":characters,"objects":objects},"mission":mission,"availableActions":actions}),
    )
}

pub fn authored_scene(state: &State) -> Result<Value, Error> {
    scene_with_provider(state, &MockSceneProvider)
}

fn canonical_scene(state: &State, input: &Value) -> Result<Value, Error> {
    let scene = state.presentation();
    let presentation:Vec<_>=scene.entries.into_iter().map(|entry|if entry.speaker_id.is_empty(){json!({"type":"narration","entryId":entry.entry_id,"text":entry.text})}else{json!({"type":"dialogue","entryId":entry.entry_id,"speakerId":entry.speaker_id,"text":entry.text})}).collect();
    let choices: Vec<_> = input["availableActions"]
        .as_array()
        .expect("validated array")
        .iter()
        .map(|a| json!({"label":a["description"],"actionId":a["id"]}))
        .collect();
    checked(
        "StoryScene",
        json!({"sceneId":scene.scene_id,"presentation":presentation,"choices":choices}),
    )
}

pub fn payload(state: &State, mode: Mode) -> Result<Value, Error> {
    let input = match mode {
        Mode::Plan => build_plan_input(state)?,
        Mode::Scene => build_scene_input(state)?,
    };
    checked(
        "StoryPayload",
        json!({"protocol":PROTOCOL,"contentVersion":wonderwilds_domain::story::CONTENT_VERSION,"stateRevision":state.revision(),"input":input}),
    )
}

#[cfg(test)]
mod tests;
