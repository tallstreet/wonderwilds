use wonderwilds_director::story::evaluation::corpus;

#[test]
fn broader_dialogue_follows_discoveries_designs_and_reachable_destinations() {
    let rows = corpus().unwrap();
    let mut checked = [0; 6];
    for row in rows {
        let input = &row.input;
        let mission = input["mission"]["missionId"].as_str().unwrap_or("");
        let status = input["mission"]["status"].as_str().unwrap_or("");
        let facts = input["context"]["memory"]["facts"].as_array().unwrap();
        let has_fact = |fact: &str| facts.iter().any(|value| value == fact);
        let entries = row.reference["presentation"].as_array().unwrap();
        let texts = entries
            .iter()
            .map(|entry| entry["text"].as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        let pip = entries
            .iter()
            .filter(|entry| entry["speakerId"] == "pip")
            .map(|entry| entry["text"].as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        if mission == "humming_hollow"
            && has_fact("Wind passing through the hollow branch makes the humming sound.")
            && !pip.is_empty()
        {
            assert!(
                !pip.contains("follow the clues"),
                "{}: {pip}",
                row.scenario_id
            );
            checked[0] += 1;
        }
        if mission == "cosy_corner"
            && status == "active"
            && has_fact("Pip's cosy corner uses leafy_nook.")
        {
            assert!(pip.contains("leafy nook"), "{}: {pip}", row.scenario_id);
            assert!(!pip.contains("We can make it"));
            assert!(!texts.contains("corner is ready"));
            checked[1] += 1;
        }
        if mission == "cosy_corner"
            && status == "active"
            && has_fact("Pip's cosy corner uses star_canopy.")
        {
            assert!(pip.contains("star canopy"), "{}: {pip}", row.scenario_id);
            assert!(!pip.contains("We can make it"));
            checked[2] += 1;
        }
        for action in input["availableActions"].as_array().unwrap() {
            if action["action"]["choiceId"] == "instrument_home"
                && action["action"]["optionId"] == "gathering"
            {
                assert_eq!(
                    action["description"],
                    "Pick up the wind instrument to take to the bridge bank."
                );
                checked[3] += 1;
            }
        }
        if mission == "brambles_visit"
            && status == "active"
            && input["mission"]["completed"].as_array().unwrap().is_empty()
            && input["context"]["locationId"] == "bridge_bank"
        {
            assert!(
                texts.contains("Walk to the workshop."),
                "{}: {texts}",
                row.scenario_id
            );
            checked[4] += 1;
        }
        if mission == "moonlight_gathering"
            && status == "active"
            && has_fact("The gathering is at far_bank.")
            && has_fact("Pip has joined the chosen gathering place.")
            && has_fact("Bramble has joined the chosen gathering place.")
            && input["context"]["locationId"] == "bridge_bank"
        {
            assert!(
                texts.contains("Walk to the far bank."),
                "{}: {texts}",
                row.scenario_id
            );
            checked[5] += 1;
        }
    }
    assert!(
        checked.into_iter().all(|count| count > 0),
        "Missing branch coverage: {checked:?}"
    );
}
