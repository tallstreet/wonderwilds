//! Internal checkpoint integrity. No caller may supply these values through HTTP.
use crate::game::Error;
use postgres::Transaction;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub fn digest(value: &Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}
pub fn envelope(version: &str, state: Value) -> Value {
    json!({"format":version,"state":state,"digest":digest(&state)})
}
pub fn unwrap(value: &Value, version: &str) -> Result<Value, Error> {
    if value["format"] != version
        || value["digest"] != digest(&value["state"])
        || value.as_object().map(|o| o.len()) != Some(3)
    {
        return Err(Error::Unavailable);
    }
    Ok(value["state"].clone())
}
pub fn save(
    tx: &mut Transaction<'_>,
    profile: &str,
    kind: &str,
    revision: u64,
    value: &Value,
) -> Result<(), Error> {
    let table = table(kind)?;
    let rev = i64::try_from(revision).map_err(|_| Error::Unavailable)?;
    tx.execute(&format!("INSERT INTO wonderwilds.{table}(profile_id,revision,snapshot,digest) VALUES($1::text::uuid,$2,$3,$4) ON CONFLICT(profile_id,revision) DO NOTHING"), &[&profile,&rev,value,&digest(value)]).map_err(|_| Error::Unavailable)?;
    // Only the current and previous verified checkpoint are hot. The origin and
    // full durable command log remain available for offline replay/rollback.
    tx.execute(
        &format!(
            "DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid AND revision < $2"
        ),
        &[&profile, &rev.saturating_sub(1)],
    )
    .map_err(|_| Error::Unavailable)?;
    Ok(())
}
pub fn previous(
    tx: &mut Transaction<'_>,
    profile: &str,
    kind: &str,
    revision: u64,
    current: &Value,
) -> Result<Option<Value>, Error> {
    let table = table(kind)?;
    let rev = i64::try_from(revision).map_err(|_| Error::Unavailable)?;
    let rows = tx.query(&format!("SELECT revision,snapshot,digest FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid ORDER BY revision DESC LIMIT 2"), &[&profile]).map_err(|_| Error::Unavailable)?;
    let Some(row) = rows.first() else {
        return Err(Error::Unavailable);
    };
    for row in &rows {
        let state: Value = row.get(1);
        if row.get::<_, String>(2) != digest(&state) {
            return Err(Error::Unavailable);
        }
    }
    if row.get::<_, i64>(0) != rev || row.get::<_, Value>(1) != *current {
        return Err(Error::Unavailable);
    }
    if let Some(row) = rows.get(1) {
        if row.get::<_, i64>(0) != rev - 1 {
            return Err(Error::Unavailable);
        }
        Ok(Some(row.get(1)))
    } else {
        Ok(None)
    }
}
pub fn origin(
    tx: &mut Transaction<'_>,
    profile: &str,
    kind: &str,
    value: &Value,
) -> Result<(), Error> {
    tx.execute("INSERT INTO wonderwilds.persistence_origins(profile_id,kind,snapshot,digest) VALUES($1::text::uuid,$2,$3,$4) ON CONFLICT DO NOTHING", &[&profile,&kind,value,&digest(value)]).map_err(|_| Error::Unavailable)?;
    Ok(())
}
fn table(kind: &str) -> Result<&'static str, Error> {
    match kind {
        "game" => Ok("game_checkpoints"),
        "story" => Ok("story_checkpoints"),
        _ => Err(Error::Unavailable),
    }
}

pub fn original(tx: &mut Transaction<'_>, profile: &str, kind: &str) -> Result<Value, Error> {
    let row=tx.query_opt("SELECT snapshot,digest FROM wonderwilds.persistence_origins WHERE profile_id=$1::text::uuid AND kind=$2", &[&profile,&kind]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
    let value: Value = row.get(0);
    if row.get::<_, String>(1) != digest(&value) {
        return Err(Error::Unavailable);
    }
    Ok(value)
}

/// Used only after a complete offline replay has verified all recorded inputs.
pub(crate) fn replace(
    tx: &mut Transaction<'_>,
    profile: &str,
    kind: &str,
    states: &[(u64, Value)],
) -> Result<(), Error> {
    let table = table(kind)?;
    tx.execute(
        &format!("DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),
        &[&profile],
    )
    .map_err(|_| Error::Unavailable)?;
    for (revision, value) in states {
        save(tx, profile, kind, *revision, value)?;
    }
    Ok(())
}

pub(crate) fn receipt(request: &Value, response: &Value, replay: &Value) -> String {
    digest(&json!([request, response, replay]))
}
/// A renewable per-profile, per-mode budget, independent of lifetime history.
/// The profile row lock serializes updates. Exact retries do not consume it.
pub(crate) fn take_budget(
    tx: &mut Transaction<'_>,
    profile: &str,
    kind: &str,
) -> Result<(), Error> {
    let row=tx.query_opt("INSERT INTO wonderwilds.command_windows(profile_id,kind,window_minute,commands) VALUES($1::text::uuid,$2,floor(extract(epoch FROM clock_timestamp())/60)::bigint,1) ON CONFLICT(profile_id,kind) DO UPDATE SET window_minute=EXCLUDED.window_minute,commands=CASE WHEN command_windows.window_minute=EXCLUDED.window_minute THEN command_windows.commands+1 ELSE 1 END WHERE command_windows.window_minute<>EXCLUDED.window_minute OR command_windows.commands<4096 RETURNING commands", &[&profile,&kind]).map_err(|_|Error::Unavailable)?;
    if row.is_none() {
        return Err(Error::RateLimited);
    }
    Ok(())
}
