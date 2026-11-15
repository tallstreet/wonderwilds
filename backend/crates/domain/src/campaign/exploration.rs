use super::*;

pub(super) fn route_open(s: &CampaignState, route: &RouteDefinition) -> bool {
    has(&s.flags, &route.requires_flags)
        || (s.location == "bridge_bank"
            && route.to == "picnic_clearing"
            && s.flags.contains("bridge_clue_complete"))
}

pub(super) fn finds(def: &CampaignDefinition) -> Vec<ActionDefinition> {
    let Some(template) = def.actions.iter().find(|a| a.id == "bridge_gather") else {
        return vec![];
    };
    ["by the old stump", "beside the picnic blanket", "under the fallen leaves"]
        .iter().enumerate().map(|(index, place)| {
            let mut action = template.clone();
            action.id = format!("find_branch_{}", index + 1);
            action.label = format!("Pick up the fallen branch {place}");
            action.location = "picnic_clearing".into();
            action.requires_flags = vec!["bridge_clue_complete".into()];
            action.forbids_flags = vec!["bridge_gather_done".into(), action.id.clone()];
            action.grants_flags = vec![action.id.clone()];
            action.grant_id.clear();
            action.rewards.clear();
            action.dialogue = vec![Dialogue { speaker: "Hazel".into(), text: "A fallen branch! Leave living branches on the trees. Find all three, then bring them back to the bridge for the woodcutter.".into() }];
            action
        }).collect()
}

/// Untimed construction and irrigation controls. The server checks every placement
/// and dependency; the browser only selects a piece or a location on the board.
pub(super) fn project_move(
    s: &mut CampaignState,
    e: &mut EncounterState,
    board: &EncounterDefinition,
    movement: &str,
) -> Result<bool, CampaignError> {
    if !movement.starts_with("bridge_") && !movement.starts_with("garden_") {
        return Ok(false);
    }
    let prefix = format!("project:{}:", e.id);
    let key = |value: &str| format!("{prefix}{value}");
    let mut complete = false;
    if e.id == "trial_bridge" {
        let part = match movement {
            "bridge_fit:near:short" => "near",
            "bridge_fit:middle:long" => "middle",
            "bridge_fit:far:short" => "far",
            "bridge_brace:near" if s.flags.contains(&key("near")) => "brace_near",
            "bridge_brace:far" if s.flags.contains(&key("far")) => "brace_far",
            "bridge_cross"
                if ["near", "middle", "far", "brace_near", "brace_far"]
                    .iter()
                    .all(|part| s.flags.contains(&key(part))) =>
            {
                complete = true;
                "crossed"
            }
            _ => return Err(CampaignError::IllegalAction),
        };
        if !s.flags.insert(key(part)) {
            return Err(CampaignError::IllegalAction);
        }
    } else if e.id == "trial_bloom" {
        match movement {
            "garden_route:north:beds"
            | "garden_route:north:collector"
            | "garden_route:south:beds"
            | "garden_route:south:collector" => {
                let parts: Vec<_> = movement.split(':').collect();
                s.flags
                    .retain(|flag| !flag.starts_with(&key(&format!("{}:", parts[1]))));
                s.flags.insert(key(&format!("{}:{}", parts[1], parts[2])));
            }
            "garden_water:north" | "garden_water:south" => {
                let bed = movement.split(':').nth(1).unwrap();
                if !s.flags.contains(&key(&format!("{bed}:beds")))
                    || !s.flags.insert(key(&format!("watered:{bed}")))
                {
                    return Err(CampaignError::IllegalAction);
                }
            }
            "garden_mulch:north" | "garden_mulch:south" => {
                let bed = movement.split(':').nth(1).unwrap();
                if !s.flags.contains(&key(&format!("watered:{bed}")))
                    || !s.flags.insert(key(&format!("mulched:{bed}")))
                {
                    return Err(CampaignError::IllegalAction);
                }
            }
            "garden_check"
                if [
                    "north:beds",
                    "south:beds",
                    "watered:north",
                    "watered:south",
                    "mulched:north",
                    "mulched:south",
                ]
                .iter()
                .all(|part| s.flags.contains(&key(part))) =>
            {
                complete = true;
            }
            _ => return Err(CampaignError::IllegalAction),
        }
    } else {
        return Err(CampaignError::IllegalAction);
    }
    e.progress = if complete {
        board.target
    } else {
        u32::try_from(
            s.flags
                .iter()
                .filter(|flag| flag.starts_with(&prefix) && !flag.ends_with(":collector"))
                .count(),
        )
        .unwrap_or(0)
        .min(board.target - 1)
    };
    // No clock, energy cost or storm-loss loop for these hands-on projects.
    e.pressure = 0;
    e.turns = 0;
    Ok(true)
}

pub(super) fn project_hint(s: &CampaignState) -> Option<String> {
    let e = s.encounter.as_ref()?;
    let has = |part: &str| s.flags.contains(&format!("project:{}:{part}", e.id));
    let steps: &[(&str, &str)] = match e.id.as_str() {
        "trial_bridge" => &[
            ("near", "bridge_fit:near:short"),
            ("middle", "bridge_fit:middle:long"),
            ("far", "bridge_fit:far:short"),
            ("brace_near", "bridge_brace:near"),
            ("brace_far", "bridge_brace:far"),
            ("crossed", "bridge_cross"),
        ],
        "trial_bloom" => &[
            ("north:beds", "garden_route:north:beds"),
            ("south:beds", "garden_route:south:beds"),
            ("watered:north", "garden_water:north"),
            ("watered:south", "garden_water:south"),
            ("mulched:north", "garden_mulch:north"),
            ("mulched:south", "garden_mulch:south"),
            ("checked", "garden_check"),
        ],
        _ => return None,
    };
    steps
        .iter()
        .find(|(part, _)| !has(part))
        .map(|(_, command)| (*command).into())
}
