//! Bounded JSONL interface for the offline synthetic scene evaluation harness.
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use wonderwilds_director::story::evaluation::{
    Candidate, Grade, Scenario, corpus, grade_candidate, validate_scenario,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MAX_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ROWS: usize = 2000;

fn read_rows<T: DeserializeOwned>(path: &str) -> Result<Vec<T>> {
    let mut bytes = vec![];
    File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Input exceeds 32 MiB".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    let mut rows = vec![];
    for (index, line) in text.lines().enumerate() {
        if line.len() > 256 * 1024 {
            return Err(format!("Line {} exceeds 256 KiB", index + 1).into());
        }
        if rows.len() >= MAX_ROWS {
            return Err("Input exceeds 2000 rows".into());
        }
        rows.push(serde_json::from_str(line).map_err(|e| format!("Line {}: {e}", index + 1))?);
    }
    if rows.is_empty() {
        return Err("Input has no records".into());
    }
    Ok(rows)
}

fn write_rows<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    // Validate/serialize before creating the file. Existing results are never replaced.
    let mut bytes = vec![];
    for row in rows {
        serde_json::to_writer(&mut bytes, row)?;
        bytes.push(b'\n');
    }
    let mut file = File::options().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("Expected export or grade")?;
    let mut options = BTreeMap::new();
    while let Some(key) = args.next() {
        if !matches!(key.as_str(), "--output" | "--corpus" | "--candidates")
            || options.contains_key(&key)
        {
            return Err(format!("Unknown or duplicate option: {key}").into());
        }
        options.insert(key, args.next().ok_or("Option needs a path")?);
    }
    let output = options
        .remove("--output")
        .ok_or("--output FILE is required")?;
    match command.as_str() {
        "export" if options.is_empty() => {
            let rows = corpus()?;
            write_rows(Path::new(&output), &rows)?;
            eprintln!("Exported {} synthetic scenarios (four related journey families).", rows.len());
        }
        "grade" => {
            let corpus_path = options.remove("--corpus").ok_or("--corpus FILE is required")?;
            let candidates_path = options.remove("--candidates").ok_or("--candidates FILE is required")?;
            if !options.is_empty() { return Err("Unexpected grade options".into()); }
            let mut scenarios = BTreeMap::new();
            for row in read_rows::<Scenario>(&corpus_path)? {
                validate_scenario(&row)?;
                if scenarios.insert(row.scenario_id.clone(), row).is_some() {
                    return Err("Duplicate scenarioId".into());
                }
            }
            let mut seen = BTreeSet::new();
            let mut grades = vec![];
            for candidate in read_rows::<Candidate>(&candidates_path)? {
                if !seen.insert((candidate.scenario_id.clone(), candidate.candidate_id.clone())) {
                    return Err("Duplicate scenarioId/candidateId pair".into());
                }
                grades.push(match scenarios.get(&candidate.scenario_id) {
                    Some(scenario) => grade_candidate(scenario, &candidate)?,
                    None => Grade { scenario_id: candidate.scenario_id, candidate_id: candidate.candidate_id,
                        passed: false, failures: vec!["unknown_scenario".into()], simulated_actions: 0 },
                });
            }
            write_rows(Path::new(&output), &grades)?;
            eprintln!("Graded {} candidates; {} hard passes. Prose quality remains ungraded.", grades.len(), grades.iter().filter(|g| g.passed).count());
        }
        _ => return Err("Usage: story_eval export --output NEW_FILE | grade --corpus FILE --candidates FILE --output NEW_FILE".into()),
    }
    Ok(())
}
