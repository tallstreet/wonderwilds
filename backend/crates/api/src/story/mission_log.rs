//! Optional synthetic-development capture; never part of a gameplay transaction.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static WRITER: Mutex<()> = Mutex::new(());

pub(super) fn record(
    path: &Path,
    kind: &str,
    request: &Value,
    output: Option<Value>,
    passed: bool,
    elapsed: Duration,
) {
    if append(path, kind, request, output, passed, elapsed).is_err() {
        eprintln!("Mission capture: could not append local record; gameplay continues");
    }
}

fn append(
    path: &Path,
    kind: &str,
    request: &Value,
    output: Option<Value>,
    passed: bool,
    elapsed: Duration,
) -> std::io::Result<()> {
    let _guard = WRITER
        .lock()
        .map_err(|_| std::io::Error::other("capture lock"))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_nanos()
        .to_string();
    let mut record = json!({"format":"mission-capture-1", "timestampUnixNs":timestamp,
        "kind":kind, "request":request, "output":output,
        "providerValidation":if passed {"passed"} else {"failed"},
        "elapsedMs":elapsed.as_millis()});
    record["id"] = json!(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&record)?)
    ));
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let mut bytes = serde_json::to_vec(&record)?;
    bytes.push(b'\n');
    file.write_all(&bytes)?;
    file.sync_data()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captures_distinct_attempts_and_keeps_failures() {
        let path = std::env::temp_dir().join(format!(
            "mission-capture-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for passed in [true, false] {
            append(
                &path,
                "neighbourhood",
                &json!({"model":"synthetic"}),
                Some(json!("invalid output\nnext line")),
                passed,
                Duration::from_millis(1),
            )
            .unwrap();
        }
        let contents = std::fs::read_to_string(&path).unwrap();
        let rows: Vec<Value> = contents
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0]["id"], rows[1]["id"]);
        assert_eq!(rows[1]["providerValidation"], "failed");
        assert_eq!(rows[1]["output"], "invalid output\nnext line");
        std::fs::remove_file(path).unwrap();
    }
}
