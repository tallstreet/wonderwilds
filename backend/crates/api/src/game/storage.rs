//! Bounded current-state format; immutable history is stored in indexed tables.
use super::*;
use crate::persistence as checkpoints;
use postgres::Transaction;

const FORMAT: &str = "game-current-1";
pub(super) fn encode(state: &Aggregate, content: &Content) -> Result<Value, Error> {
    Ok(checkpoints::envelope(
        FORMAT,
        json!({"contentVersion":content.version(),"contentFingerprint":checkpoints::digest(content.canonical_source()),"rulesVersion":engine::RULES_VERSION,"aggregate":state.compact()?}),
    ))
}
fn decode(value: &Value, content: &Content) -> Result<Aggregate, Error> {
    let body = checkpoints::unwrap(value, FORMAT)?;
    if body["contentVersion"] != content.version()
        || body["rulesVersion"] != engine::RULES_VERSION
        || body["contentFingerprint"] != checkpoints::digest(content.canonical_source())
    {
        return Err(Error::Unavailable);
    }
    let state: Aggregate =
        serde_json::from_value(body["aggregate"].clone()).map_err(|_| Error::Unavailable)?;
    if encode(&state, content)? != *value
        || state.compact_policy.as_deref() != Some(Aggregate::COMPACT_POLICY)
    {
        return Err(Error::Unavailable);
    }
    state.snapshot(content)?;
    Ok(state)
}
fn reconstruction(saved: &SavedEncounter, content: &Content) -> Result<Value, Error> {
    let (_, encounter) =
        authored_encounter(&saved.skill, saved.seed, &saved.id).map_err(|_| Error::Unavailable)?;
    Ok(
        json!({"format":"authored-observation-1","contentVersion":content.version(),"contentFingerprint":checkpoints::digest(content.canonical_source()),"policyVersion":Aggregate::COMPACT_POLICY,"parameterFingerprint":digest_id(&serde_json::to_string(&encounter.parameters).map_err(|_|Error::Unavailable)?),"novelTransfer":false}),
    )
}
fn archive_encounters(
    tx: &mut Transaction<'_>,
    profile: &str,
    state: &Aggregate,
) -> Result<(), Error> {
    for (id, encounter) in &state.encounters {
        let value = serde_json::to_value(encounter).map_err(|_| Error::Unavailable)?;
        tx.execute("INSERT INTO wonderwilds.game_encounters(profile_id,encounter_id,encounter) VALUES($1::text::uuid,$2,$3) ON CONFLICT(profile_id,encounter_id) DO UPDATE SET encounter=EXCLUDED.encounter", &[&profile,id,&value]).map_err(|_| Error::Unavailable)?;
    }
    Ok(())
}
pub(super) fn load(
    tx: &mut Transaction<'_>,
    profile: &str,
    content: &Content,
) -> Result<Aggregate, Error> {
    let saved = tx
        .query_opt(
            "SELECT snapshot FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",
            &[&profile],
        )
        .map_err(|_| Error::Unavailable)?;
    let value = saved.map(|r| r.get::<_, Value>(0));
    let profile_revision: i64 = tx
        .query_one(
            "SELECT revision FROM wonderwilds.profiles WHERE id=$1::text::uuid",
            &[&profile],
        )
        .map_err(|_| Error::Unavailable)?
        .get(0);
    if value.is_none() && profile_revision != 0 {
        return Err(Error::Unavailable);
    }
    if let Some(value) = &value
        && value.get("format").is_some()
    {
        let state = decode(value, content)?;
        if state.revision as i64 != profile_revision {
            return Err(Error::Unavailable);
        }
        let previous = checkpoints::previous(tx, profile, "game", state.revision, value)?;
        let has_previous = previous.is_some();
        if let Some(previous) = previous {
            let mut prior = decode(&previous, content)?;
            let row = tx.query_opt("SELECT request,response,replay,digest FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND revision=$2", &[&profile,&(state.revision as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
            let request: Value = row.get(0);
            let response: Value = row.get(1);
            let replay: Value = row.get(2);
            if row.get::<_, String>(3) != checkpoints::receipt(&request, &response, &replay) {
                return Err(Error::Unavailable);
            }
            let result = prior
                .execute(
                    &request,
                    content,
                    replay["now"].as_u64().ok_or(Error::Unavailable)?,
                    replay["session"].as_str().ok_or(Error::Unavailable)?,
                )
                .map_err(|_| Error::Unavailable)?;
            if result != response || encode(&prior, content)? != *value {
                return Err(Error::Unavailable);
            }
        }
        if !has_previous {
            let origin = read_origin(tx, profile, content)?;
            if encode(&origin, content)? != *value {
                return Err(Error::Unavailable);
            }
        }
        let events: i64=tx.query_one("SELECT coalesce(max(sequence),0) FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_|Error::Unavailable)?.get(0);
        let evidence: i64=tx.query_one("SELECT coalesce(max(sequence),0) FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_|Error::Unavailable)?.get(0);
        if events != state.domain.event_sequence() as i64
            || evidence != state.evidence_sequence as i64
        {
            return Err(Error::Unavailable);
        }
        return Ok(state);
    }
    // Legacy migration is owner-locked and atomic. Validate the old public and
    // learning projections before archiving anything or changing the format.
    let legacy: Aggregate = match &value {
        Some(v) => serde_json::from_value(v.clone()).map_err(|_| Error::Unavailable)?,
        None => Aggregate::new(content),
    };
    let before = legacy.snapshot(content)?;
    if legacy.receipts.len() as u64 != legacy.revision || legacy.revision as i64 != profile_revision
    {
        return Err(Error::Unavailable);
    }
    let compact = legacy.compact()?;
    if compact.snapshot(content)? != before {
        return Err(Error::Unavailable);
    }
    for evidence in &legacy.evidence {
        let row = tx.query_opt("SELECT evidence FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&(evidence.server_sequence as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
        if row.get::<_, Value>(0)
            != serde_json::to_value(evidence).map_err(|_| Error::Unavailable)?
        {
            return Err(Error::Unavailable);
        }
        let saved = legacy
            .encounters
            .get(&evidence.encounter_id)
            .ok_or(Error::Unavailable)?;
        tx.execute("UPDATE wonderwilds.learning_events SET reconstruction=$3 WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&(evidence.server_sequence as i64),&reconstruction(saved,content)?]).map_err(|_|Error::Unavailable)?;
    }
    for event in &legacy.events {
        let row = tx.query_opt("SELECT event FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&(event.sequence as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
        if row.get::<_, Value>(0) != serde_json::to_value(event).map_err(|_| Error::Unavailable)? {
            return Err(Error::Unavailable);
        }
    }
    for (id, receipt) in &legacy.receipts {
        wire::validate("GameCommand", &receipt.request).map_err(|_| Error::Unavailable)?;
        wire::validate("GameSnapshot", &receipt.response).map_err(|_| Error::Unavailable)?;
        if receipt.request["commandId"] != *id {
            return Err(Error::Unavailable);
        }
        let revision = receipt.response["revision"]
            .as_i64()
            .ok_or(Error::Unavailable)?;
        if revision < 1
            || revision > legacy.revision as i64
            || receipt.request["expectedRevision"].as_i64() != Some(revision - 1)
        {
            return Err(Error::Unavailable);
        }
        tx.execute("INSERT INTO wonderwilds.command_receipts(profile_id,command_id,revision,request,response,replay,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6,$7)", &[&profile,id,&revision,&receipt.request,&receipt.response,&json!({"legacy":true}),&checkpoints::receipt(&receipt.request,&receipt.response,&json!({"legacy":true}))]).map_err(|_|Error::Unavailable)?;
    }
    let receipts = legacy.domain.archived_receipts();
    for (id, record) in receipts.as_object().ok_or(Error::Unavailable)? {
        tx.execute("INSERT INTO wonderwilds.domain_command_receipts(profile_id,command_id,record) VALUES($1::text::uuid,$2,$3)", &[&profile,id,record]).map_err(|_|Error::Unavailable)?;
    }
    archive_encounters(tx, profile, &legacy)?;
    checkpoints::origin(
        tx,
        profile,
        "game",
        &json!({"format":"game-origin-1","contentFingerprint":checkpoints::digest(content.canonical_source()),"aggregate":legacy}),
    )?;
    persist_current(tx, profile, &compact, content)?;
    Ok(compact)
}
fn persist_current(
    tx: &mut Transaction<'_>,
    profile: &str,
    state: &Aggregate,
    content: &Content,
) -> Result<(), Error> {
    let value = encode(state, content)?;
    if value.to_string().len() > 65536 {
        return Err(Error::RateLimited);
    }
    checkpoints::save(tx, profile, "game", state.revision, &value)?;
    tx.execute("INSERT INTO wonderwilds.progress(profile_id,snapshot) VALUES($1::text::uuid,$2) ON CONFLICT(profile_id) DO UPDATE SET snapshot=EXCLUDED.snapshot", &[&profile,&value]).map_err(|_|Error::Unavailable)?;
    tx.execute(
        "UPDATE wonderwilds.profiles SET revision=$2 WHERE id=$1::text::uuid",
        &[&profile, &(state.revision as i64)],
    )
    .map_err(|_| Error::Unavailable)?;
    Ok(())
}
pub(super) fn commit(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    state: &Aggregate,
    content: &Content,
    request: &Value,
    response: &Value,
    now: u64,
) -> Result<(), Error> {
    let profile = identity.profile_id.as_str();
    let session = identity.session_id.as_str();
    archive_encounters(tx, profile, state)?;
    for evidence in &state.evidence {
        let saved = state
            .encounters
            .get(&evidence.encounter_id)
            .ok_or(Error::Unavailable)?;
        let value = serde_json::to_value(evidence).map_err(|_| Error::Unavailable)?;
        tx.execute("INSERT INTO wonderwilds.learning_events(profile_id,sequence,evidence,reconstruction) VALUES($1::text::uuid,$2,$3,$4)", &[&profile,&(evidence.server_sequence as i64),&value,&reconstruction(saved,content)?]).map_err(|_|Error::Unavailable)?;
    }
    for event in &state.events {
        let value = serde_json::to_value(event).map_err(|_| Error::Unavailable)?;
        for table in ["game_events", "game_outbox"] {
            tx.execute(&format!("INSERT INTO wonderwilds.{table}(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3)"), &[&profile,&(event.sequence as i64),&value]).map_err(|_|Error::Unavailable)?;
        }
        if let engine::Outcome::RewardGranted {
            reward_id,
            quest_id,
        } = &event.outcome
        {
            tx.execute("INSERT INTO wonderwilds.reward_grants(profile_id,reward_definition,quest_instance) VALUES($1::text::uuid,$2,$3)", &[&profile,reward_id,quest_id]).map_err(|_|Error::Unavailable)?;
        }
    }
    for record in &state.domain_records {
        let id = record["command"]["commandId"]
            .as_str()
            .ok_or(Error::Unavailable)?;
        tx.execute("INSERT INTO wonderwilds.domain_command_receipts(profile_id,command_id,record) VALUES($1::text::uuid,$2,$3)", &[&profile,&id,record]).map_err(|_|Error::Unavailable)?;
    }
    tx.execute("INSERT INTO wonderwilds.command_receipts(profile_id,command_id,revision,request,response,replay,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6,$7)", &[&profile,&request["commandId"].as_str().ok_or(Error::InvalidInput)?,&(state.revision as i64),request,response,&json!({"now":now,"session":session}),&checkpoints::receipt(request,response,&json!({"now":now,"session":session}))]).map_err(|_|Error::Unavailable)?;
    persist_current(tx, profile, state, content)
}

/// Offline audit/recovery path. Normal reads never scan these histories.
/// Rebuild from the validated migration origin and recorded server clock/session,
/// compare every original response, and independently recompute learning history.
pub(super) fn rebuild(
    tx: &mut Transaction<'_>,
    profile: &str,
    content: &Content,
    restore: bool,
) -> Result<Aggregate, Error> {
    let legacy = read_origin(tx, profile, content)?;
    legacy.snapshot(content)?;
    for (id, receipt) in &legacy.receipts {
        let row=tx.query_opt("SELECT request,response,replay,digest FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2", &[&profile,id]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
        if row.get::<_, Value>(0) != receipt.request
            || row.get::<_, Value>(1) != receipt.response
            || row.get::<_, String>(3)
                != checkpoints::receipt(&receipt.request, &receipt.response, &row.get(2))
        {
            return Err(Error::Unavailable);
        }
    }
    for e in &legacy.evidence {
        let row=tx.query_opt("SELECT evidence,reconstruction FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&(e.server_sequence as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
        if row.get::<_, Value>(0) != serde_json::to_value(e).map_err(|_| Error::Unavailable)?
            || row.get::<_, Value>(1)
                != reconstruction(
                    legacy
                        .encounters
                        .get(&e.encounter_id)
                        .ok_or(Error::Unavailable)?,
                    content,
                )?
        {
            return Err(Error::Unavailable);
        }
    }
    for (id, record) in legacy
        .domain
        .archived_receipts()
        .as_object()
        .ok_or(Error::Unavailable)?
    {
        let row=tx.query_opt("SELECT record FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2", &[&profile,id]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
        if row.get::<_, Value>(0) != *record {
            return Err(Error::Unavailable);
        }
        for event in record["events"].as_array().ok_or(Error::Unavailable)? {
            let row=tx.query_opt("SELECT event FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&event["sequence"].as_i64().ok_or(Error::Unavailable)?]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
            if row.get::<_, Value>(0) != *event {
                return Err(Error::Unavailable);
            }
        }
    }
    let mut state = legacy.compact()?;
    let mut verified = vec![(state.revision, encode(&state, content)?)];
    loop {
        let rows=tx.query("SELECT revision,request,response,replay,digest FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND revision>$2 ORDER BY revision LIMIT 256", &[&profile,&(state.revision as i64)]).map_err(|_|Error::Unavailable)?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            if row.get::<_, i64>(0) != state.revision as i64 + 1 {
                return Err(Error::Unavailable);
            }
            let request: Value = row.get(1);
            let expected: Value = row.get(2);
            let replay: Value = row.get(3);
            if row.get::<_, String>(4) != checkpoints::receipt(&request, &expected, &replay) {
                return Err(Error::Unavailable);
            }
            let response = state
                .execute(
                    &request,
                    content,
                    replay["now"].as_u64().ok_or(Error::Unavailable)?,
                    replay["session"].as_str().ok_or(Error::Unavailable)?,
                )
                .map_err(|_| Error::Unavailable)?;
            if response != expected {
                return Err(Error::Unavailable);
            }
            for e in &state.evidence {
                let row=tx.query_opt("SELECT evidence,reconstruction FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&(e.server_sequence as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
                if row.get::<_, Value>(0)
                    != serde_json::to_value(e).map_err(|_| Error::Unavailable)?
                    || row.get::<_, Value>(1)
                        != reconstruction(
                            state
                                .encounters
                                .get(&e.encounter_id)
                                .ok_or(Error::Unavailable)?,
                            content,
                        )?
                {
                    return Err(Error::Unavailable);
                }
            }
            for e in &state.events {
                let row=tx.query_opt("SELECT event FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid AND sequence=$2", &[&profile,&(e.sequence as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
                if row.get::<_, Value>(0)
                    != serde_json::to_value(e).map_err(|_| Error::Unavailable)?
                {
                    return Err(Error::Unavailable);
                }
            }
            for record in &state.domain_records {
                let id = record["command"]["commandId"]
                    .as_str()
                    .ok_or(Error::Unavailable)?;
                let row=tx.query_opt("SELECT record FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2", &[&profile,&id]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
                if row.get::<_, Value>(0) != *record {
                    return Err(Error::Unavailable);
                }
            }
            state = state.compact()?;
            verified.push((state.revision, encode(&state, content)?));
            if verified.len() > 2 {
                verified.remove(0);
            }
        }
    }
    let revision: i64 = tx
        .query_one(
            "SELECT revision FROM wonderwilds.profiles WHERE id=$1::text::uuid",
            &[&profile],
        )
        .map_err(|_| Error::Unavailable)?
        .get(0);
    if revision != state.revision as i64 {
        return Err(Error::Unavailable);
    }
    let events:i64=tx.query_one("SELECT coalesce(max(sequence),0) FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_|Error::Unavailable)?.get(0);
    let evidence:i64=tx.query_one("SELECT coalesce(max(sequence),0) FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_|Error::Unavailable)?.get(0);
    let domain_receipts:i64=tx.query_one("SELECT count(*) FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_|Error::Unavailable)?.get(0);
    if events != state.domain.event_sequence() as i64
        || evidence != state.evidence_sequence as i64
        || domain_receipts != state.domain.revision() as i64
    {
        return Err(Error::Unavailable);
    }
    // Independent full-history projection is intentional in this offline tool.
    let mut full = state.clone();
    full.compact_policy = None;
    for row in tx.query("SELECT encounter_id,encounter FROM wonderwilds.game_encounters WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_|Error::Unavailable)? {
        full.encounters.insert(row.get(0),serde_json::from_value(row.get(1)).map_err(|_|Error::Unavailable)?);
    }
    for row in tx.query("SELECT evidence FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid ORDER BY sequence", &[&profile]).map_err(|_|Error::Unavailable)? {
        full.evidence.push(serde_json::from_value(row.get(0)).map_err(|_|Error::Unavailable)?);
    }
    if full.snapshot(content)? != state.snapshot(content)? {
        return Err(Error::Unavailable);
    }
    if restore {
        checkpoints::replace(tx, profile, "game", &verified)?;
    }
    Ok(state)
}
pub(super) fn recover(
    tx: &mut Transaction<'_>,
    profile: &str,
    content: &Content,
) -> Result<Value, Error> {
    let state = rebuild(tx, profile, content, true)?;
    // Does not restore an old backup: reconstruction must reach the profile's
    // latest committed revision. Missing history prevents recovery.
    persist_current(tx, profile, &state, content)?;
    state.snapshot(content)
}

fn read_origin(
    tx: &mut Transaction<'_>,
    profile: &str,
    content: &Content,
) -> Result<Aggregate, Error> {
    let value = checkpoints::original(tx, profile, "game")?;
    if value["format"] != "game-origin-1"
        || value["contentFingerprint"] != checkpoints::digest(content.canonical_source())
    {
        return Err(Error::Unavailable);
    }
    serde_json::from_value(value["aggregate"].clone()).map_err(|_| Error::Unavailable)
}
