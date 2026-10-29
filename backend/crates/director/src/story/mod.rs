//! Deterministic authored story projections; no provider, identity or learning data.
//!
//! See README.md for scope. Runtime state is the only evidence source. The small
//! catalogue below describes the bound actions implemented by domain::story.
use serde_json::{Value, json};
use wonderwilds_contracts::validate;
use wonderwilds_domain::story::{Action, Location, MissionStatus, State};

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
fn narrative() -> Value {
    json!({"premise":"Pip wants to explore the far bank with the player. Help make a way across together.","participantIds":["pip","woodcutter"],"payoff":"Cross the repaired bridge with Pip and celebrate reaching the far bank."})
}
fn character(state: &State, id: &str) -> Value {
    match id {
        "pip" => {
            json!({"id":"pip","description":"A curious and patient friend.","relationship":if state.crossed_with_pip(){"The player and Pip crossed the bridge together."}else if !state.milestones().is_empty(){"The player accepted a mission to cross the stream with Pip."}else{"No prior shared mission is recorded."}})
        }
        _ => {
            json!({"id":"woodcutter","description":"A patient neighbour who shapes wood.","relationship":if state.met_woodcutter() {"The game has presented the woodcutter's introduction."} else {"No recorded presentation of the woodcutter's introduction."}})
        }
    }
}
fn context(state: &State) -> Value {
    let mut inventory = Vec::new();
    if state.wood() > 0 {
        inventory.push(json!({"itemId":"wood","quantity":state.wood()}));
    }
    if state.planks() > 0 {
        inventory.push(json!({"itemId":"planks","quantity":state.planks()}));
    }
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
    let recent_missions: Vec<&str> = if state.mission_status() == MissionStatus::Completed {
        vec!["Help Pip cross the stream: prepare a route, then cross together; completed."]
    } else {
        vec![]
    };
    json!({"trigger":trigger,"player":{"language":"short_simple","interests":[],"avoid":[]},"locationId":state.location().as_str(),"inventory":inventory,"memory":{"facts":facts,"recentEvents":recent,"recentMissions":recent_missions}})
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
        json!({"opportunityId":"pips_bridge","goal":"Help Pip cross the stream.","narrative":narrative(),"milestones":milestone_definitions(state).into_iter().map(|(id,label)|json!({"label":label,"conditionId":id})).collect::<Vec<_>>()}),
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
    let conditions = [
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
    ];
    let opportunities = if state.mission_status() == MissionStatus::Completed {
        vec![]
    } else {
        vec![
            json!({"id":"pips_bridge","description":"Help Pip reach the far bank together, repairing the bridge if needed.","outcomeId":"bridge_and_crossing","outcome":"The bridge is repaired and the player has crossed it with Pip.","conditions":conditions.iter().map(|(id,meaning)|json!({"id":id,"meaning":meaning})).collect::<Vec<_>>()}),
        ]
    };
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
        "characters":characters},"opportunities":opportunities,"capabilities":capabilities(state)}),
    )
}

fn capabilities(state: &State) -> Vec<Value> {
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
    if !state.bridge_repaired() {
        for (source, at) in [("fallen_wood", "bridge_bank"), ("spare_wood", "workshop")] {
            let count = state.source_remaining(source).unwrap_or(0);
            if count > 0 {
                result.push(json!({"action":{"type":"collect","sourceId":source,"itemId":"wood","quantity":count},"at":at,"requires":["Active mission",format!("Source has {count} wood")],"effects":[format!("Source loses {count} wood"),format!("Player gains {count} wood")]}));
            }
        }
        result.push(json!({"action":{"type":"exchange","characterId":"woodcutter","recipeId":"wood_to_planks"},"at":"workshop","requires":["Active mission","Player has 3 wood","Bridge broken"],"effects":["Consume 3 wood","Grant 3 planks"]}));
        result.push(json!({"action":{"type":"assemble","objectId":"bridge","recipeId":"repair_bridge"},"at":"bridge_bank","requires":["Active mission","Player has 3 planks","Bridge broken"],"effects":["Consume 3 planks","Bridge becomes repaired"]}));
    }
    if !state.crossed_with_pip() {
        result.push(json!({"action":{"type":"accompany","characterId":"pip","destinationId":"far_bank"},"at":"bridge_bank","requires":["Active mission","Bridge repaired","Pip present"],"effects":["Player and Pip arrive at far bank","Record shared crossing"]}));
    }
    result
}

fn action_description(state: &State, action: &Action) -> String {
    match action {
        Action::AcceptMission if state.mission_status() == MissionStatus::Abandoned => {
            "Keep helping Pip with the bridge.".into()
        }
        Action::AcceptMission => "Help Pip cross the stream.".into(),
        Action::CancelMission => "Take a break from helping Pip.".into(),
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
            if source_id == "fallen_wood" {
                format!("Collect {quantity} pieces of wood from fallen branches.")
            } else {
                format!("Collect {quantity} pieces of spare wood.")
            }
        }
        Action::Exchange { .. } => {
            "Ask the woodcutter to turn 3 pieces of wood into 3 planks.".into()
        }
        Action::Assemble { .. } => "Use 3 planks to repair the bridge.".into(),
        Action::Accompany { .. } => "Cross the repaired bridge together with Pip.".into(),
        Action::Acknowledge { .. } => "Continue.".into(),
    }
}
fn action_handle(state: &State, index: usize) -> String {
    format!("story-{}-{index}", state.revision())
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
    let objects = match state.location() {
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
        json!({"goal":"Help Pip cross the stream.","status":state.mission_status().as_str(),"narrative":narrative(),"completed":completed,"current":pending.first(),"remaining":pending.iter().skip(1).collect::<Vec<_>>()})
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
    let input = build_scene_input(state)?;
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
