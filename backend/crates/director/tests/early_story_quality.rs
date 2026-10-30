//! Regression coverage for state-aware mock prose found by the independent review.
use serde_json::Value;
use wonderwilds_director::story::evaluation::corpus;

fn quantity(input: &Value, item: &str) -> u64 {
    input["context"]["inventory"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["itemId"] == item)
        .and_then(|entry| entry["quantity"].as_u64())
        .unwrap_or(0)
}

fn text(scene: &Value) -> String {
    scene["presentation"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["text"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn early_scenes_acknowledge_inventory_completed_banks_and_workshop_history() {
    let rows = corpus().unwrap();
    let mut partial_seen = [false; 3];
    let mut near_lit_seen = false;
    let mut far_lit_seen = false;
    let mut workshop_offer_seen = false;
    let mut workshop_pause_seen = false;
    let mut repaired_offer_seen = false;
    for row in rows {
        let input = &row.input;
        let scene = text(&row.reference);
        let mission = input["mission"]["missionId"].as_str();
        let status = input["mission"]["status"].as_str();
        let facts = input["context"]["memory"]["facts"].as_array().unwrap();
        let has_fact = |wanted: &str| facts.iter().any(|fact| fact == wanted);
        let actions = input["availableActions"].as_array().unwrap();
        let has_action = |kind: &str| actions.iter().any(|a| a["action"]["type"] == kind);
        for (index, (mission_id, item)) in [
            ("pips_bridge", "wood"),
            ("pips_picnic", "berries"),
            ("welcome_lights", "glow_pebbles"),
        ]
        .into_iter()
        .enumerate()
        {
            if mission == Some(mission_id)
                && status == Some("active")
                && quantity(input, item) == 1
                && has_action("collect")
            {
                assert!(scene.contains("You have 1"), "{}: {scene}", row.scenario_id);
                assert!(scene.contains("2 more"), "{}: {scene}", row.scenario_id);
                partial_seen[index] = true;
            }
        }
        if mission == Some("welcome_lights") && has_action("assemble") {
            if has_fact("Two lanterns light the near bank.") {
                assert!(scene.contains("near bank is already lit"));
                assert!(scene.contains("last lantern here on the far bank"));
                near_lit_seen = true;
            }
            if has_fact("One lantern lights the far bank.") {
                assert!(scene.contains("far bank is already lit"));
                assert!(scene.contains("remaining two lanterns here on the near bank"));
                far_lit_seen = true;
            }
        }
        if input["context"]["locationId"] == "workshop"
            && has_fact("The game has presented the woodcutter's introduction.")
            && (input["mission"].is_null()
                || (mission == Some("pips_bridge") && status == Some("abandoned")))
        {
            assert_eq!(row.reference["presentation"].as_array().unwrap().len(), 1);
            assert!(!scene.contains("I can help shape wood"));
            workshop_offer_seen |= input["mission"].is_null();
            workshop_pause_seen |= status == Some("abandoned");
        }
        if input["mission"].is_null() && has_fact("The bridge is repaired.") {
            assert!(
                scene.contains("bridge is ready") || scene.contains("bridge is already repaired")
            );
            assert!(!scene.contains("make a way across"));
            repaired_offer_seen = true;
        }
    }
    assert!(partial_seen.into_iter().all(|seen| seen));
    assert!(near_lit_seen && far_lit_seen);
    assert!(workshop_offer_seen && workshop_pause_seen && repaired_offer_seen);
}
