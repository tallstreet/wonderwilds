//! Wonderwilds content boundary. Implementation follows the WW dependency gates.

/// Load immutable authored campaign data and reject unsupported rules/references.
pub fn load_campaign_json(
    json: &str,
) -> Result<wonderwilds_domain::campaign::CampaignDefinition, String> {
    let definition =
        serde_json::from_str(json).map_err(|error| format!("campaign JSON: {error}"))?;
    wonderwilds_domain::campaign::validate(&definition)
        .map_err(|error| format!("campaign validation: {error:?}"))?;
    Ok(definition)
}

pub fn campaign() -> Result<wonderwilds_domain::campaign::CampaignDefinition, String> {
    load_campaign_json(include_str!("../../../../content/campaign/runtime.v1.json"))
}

/// Immutable campaign definitions remain readable after new authored releases.
pub fn campaign_version(
    version: &str,
) -> Result<wonderwilds_domain::campaign::CampaignDefinition, String> {
    match version {
        "1.0.0" => load_campaign_json(include_str!(
            "../../../../content/campaign/runtime.initial.v1.json"
        )),
        "1.1.0" => campaign(),
        _ => Err("unsupported campaign content version".into()),
    }
}
