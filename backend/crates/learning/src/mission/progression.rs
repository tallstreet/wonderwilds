//! Draft NSW practice progression, independent of academic mastery and story access.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Practice {
    /// Latest first-check result per distinct item, bounded to ten.
    pub checks: BTreeMap<String, Check>,
    /// Unique finished items, including explicitly supported exploration.
    pub explored: BTreeSet<String>,
    pub struggles: u8,
    pub review_after: u64,
    pub last_completion: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Check {
    pub correct: bool,
    pub sequence: u64,
    pub session: String,
}

pub fn record(state: &mut MissionState, o: &MissionObservation) {
    let e = &o.evidence;
    if e.provenance != Provenance::ServerVerified {
        return;
    }
    let p = state.practice.entry(e.skill_id.clone()).or_default();
    if e.event_kind == EventKind::AnswerCheck && e.first_check {
        if e.result == ResultKind::Incorrect {
            p.struggles = p.struggles.saturating_add(1).min(2);
            if p.struggles == 2 {
                p.review_after = e.server_sequence;
            }
        } else if e.result == ResultKind::Correct {
            p.struggles = 0;
        }
        if e.result != ResultKind::NotAssessed
            && e.support.hint_count == 0
            && !e.support.worked_example
            && !e.support.read_aloud
        {
            p.checks.insert(
                o.parameter_fingerprint.clone(),
                Check {
                    correct: e.result == ResultKind::Correct,
                    sequence: e.server_sequence,
                    session: e.session_id.clone(),
                },
            );
            if p.checks.len() > 10 {
                let oldest = p
                    .checks
                    .iter()
                    .min_by_key(|(_, c)| c.sequence)
                    .unwrap()
                    .0
                    .clone();
                p.checks.remove(&oldest);
            }
        }
    }
    if o.activity_completed {
        if p.explored.len() < 10 {
            p.explored.insert(o.parameter_fingerprint.clone());
        }
        p.last_completion = e.server_sequence;
    }
}

pub fn ready(p: &Practice) -> bool {
    p.checks.len() == 10 && p.checks.values().filter(|c| c.correct).count() >= 8
}

pub fn completed(skill: &str, state: &MissionState) -> bool {
    state
        .practice
        .get(skill)
        .is_some_and(|p| ready(p) || p.explored.len() >= 10)
}

pub fn summary(skill: &str, state: &MissionState) -> SkillSummary {
    let p = state.practice.get(skill).cloned().unwrap_or_default();
    let ready = ready(&p);
    SkillSummary {
        skill_id: skill.into(),
        status: if ready { Status::PracticeReady } else if !p.checks.is_empty() || !p.explored.is_empty() { Status::Practising } else { Status::Unseen },
        eligible_checks: p.checks.len(), correct_checks: p.checks.values().filter(|c| c.correct).count(),
        distinct_variants: p.checks.len(), distinct_contexts: usize::from(!p.checks.is_empty()), distinct_representations: usize::from(!p.checks.is_empty()),
        distinct_sessions: p.checks.values().map(|c| &c.session).collect::<BTreeSet<_>>().len(),
        blockers: vec!["Draft practice evidence; educator calibration and mastery review remain pending.".into()],
        next_step: if p.struggles >= 2 { "Revisit a prerequisite, then return to this skill." } else if ready { "Ready for related practice after ten distinct checks; this is not certified mastery." } else if p.explored.len() >= 10 { "Explored ten items, including support. Related practice is available; ability is not established by support." } else { "Try ten different items. Help and supported exploration remain available." }.into(),
    }
}

pub fn frontier(
    state: &MissionState,
    subject: Option<&str>,
) -> Result<Vec<SkillProjection>, MissionError> {
    let graph = CurriculumGraph::active().map_err(|_| MissionError::InvalidState)?;
    graph
        .nodes()
        .filter(|n| {
            subject.is_none_or(|s| s == n.subject)
                && !completed(&n.id, state)
                && n.prerequisites.iter().all(|p| completed(p, state))
        })
        .map(|n| projection(n, state))
        .collect()
}

pub fn select(
    state: &MissionState,
    subject: Option<&str>,
    seed: u64,
) -> Result<Option<SkillProjection>, MissionError> {
    let graph = CurriculumGraph::active().map_err(|_| MissionError::InvalidState)?;
    // Revision is a scheduling action, never a reverse edge or revocation of earned progress.
    for node in graph
        .nodes()
        .filter(|n| subject.is_none_or(|s| s == n.subject))
    {
        if let Some(p) = state.practice.get(&node.id).filter(|p| p.struggles >= 2) {
            if let Some(parent) = node
                .prerequisites
                .iter()
                .filter_map(|id| graph.get(id))
                .find(|n| {
                    state
                        .practice
                        .get(&n.id)
                        .is_none_or(|v| v.last_completion <= p.review_after)
                })
            {
                return Ok(Some(projection(parent, state)?));
            }
            if !completed(&node.id, state) {
                return Ok(Some(projection(node, state)?));
            }
        }
    }
    let mut candidates = frontier(state, subject)?;
    if state.practice.is_empty()
        && subject.is_none_or(|s| s == "maths")
        && let Some(root) = candidates.iter().find(|s| s.skill_id == "M-K-001")
    {
        return Ok(Some(root.clone()));
    }
    let year = |id: &str| {
        id.split('-')
            .nth(1)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0)
    };
    if let Some(earliest) = candidates.iter().map(|s| year(&s.skill_id)).min() {
        candidates.retain(|s| year(&s.skill_id) == earliest);
    }
    // Continue a started skill; rotate items rather than abandoning it after one success.
    if candidates
        .iter()
        .any(|s| state.practice.contains_key(&s.skill_id))
    {
        candidates.retain(|s| state.practice.contains_key(&s.skill_id));
    }
    Ok(if candidates.is_empty() {
        None
    } else {
        Some(candidates[seed as usize % candidates.len()].clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wonderwilds_contracts::{LearningEvidenceSchemaVersion, SupportState};
    fn event(skill: &str, n: u64, correct: bool, supported: bool) -> MissionObservation {
        MissionObservation {
            activity_completed: correct || supported,
            parameter_fingerprint: format!("{skill}-item-{n}"),
            evidence: LearningEvidence {
                schema_version: LearningEvidenceSchemaVersion::V10,
                event_id: format!("e{n}"),
                encounter_id: format!("a{n}"),
                skill_id: skill.into(),
                variant_id: format!("v{n}"),
                item_family: "question".into(),
                context_id: "practice".into(),
                representation: "number_entry".into(),
                session_id: format!("session-{}", n % 2),
                server_sequence: n,
                provenance: Provenance::ServerVerified,
                event_kind: if supported {
                    EventKind::WorkedExample
                } else {
                    EventKind::AnswerCheck
                },
                result: if supported {
                    ResultKind::NotAssessed
                } else if correct {
                    ResultKind::Correct
                } else {
                    ResultKind::Incorrect
                },
                first_check: !supported,
                support: SupportState {
                    hint_count: 0,
                    worked_example: supported,
                    read_aloud: false,
                },
            },
        }
    }
    #[test]
    fn replacement_has_every_skill_and_ten_distinct_items() {
        let graph = CurriculumGraph::active().unwrap();
        assert_eq!(graph.nodes().count(), 736);
        assert_eq!(graph.topological_order().unwrap().len(), 736);
        for n in graph.nodes() {
            assert!(!n.learning_resources.is_empty());
            let fingerprints = (0..10)
                .map(|seed| {
                    practice_activity(&n.id, seed, "coverage")
                        .unwrap()
                        .canonical_fingerprint()
                        .to_owned()
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(fingerprints.len(), 10, "{}", n.id);
        }
    }
    #[test]
    fn ten_items_unlock_children_and_supported_work_is_not_readiness() {
        let mut state = MissionState::default();
        for n in 1..=9 {
            super::super::record(&mut state, event("M-K-001", n, true, false)).unwrap();
        }
        assert!(!completed("M-K-001", &state));
        super::super::record(&mut state, event("M-K-001", 10, true, false)).unwrap();
        assert!(completed("M-K-001", &state));
        assert_eq!(summary("M-K-001", &state).status, Status::PracticeReady);
        let mut supported = MissionState::default();
        let mut history = Vec::new();
        for n in 1..=10 {
            let e = event("M-K-001", n, false, true);
            history.push(e.clone());
            super::super::record(&mut supported, e).unwrap();
        }
        assert!(completed("M-K-001", &supported));
        assert_eq!(summary("M-K-001", &supported).eligible_checks, 0);
        assert_ne!(summary("M-K-001", &supported).status, Status::PracticeReady);
        assert_eq!(replay(&history).unwrap(), supported);
    }
    #[test]
    fn every_replacement_skill_is_reachable_and_policy_storage_stays_bounded() {
        let graph = CurriculumGraph::active().unwrap();
        let mut state = MissionState::default();
        let mut sequence = 0;
        for node in graph.topological_order().unwrap() {
            assert!(node.prerequisites.iter().all(|p| completed(p, &state)));
            for _ in 0..10 {
                sequence += 1;
                super::super::record(&mut state, event(&node.id, sequence, true, false)).unwrap();
            }
            assert!(completed(&node.id, &state));
        }
        assert!(select(&state, None, 0).unwrap().is_none());
        assert_eq!(state.practice.len(), 736);
        assert!(serde_json::to_vec(&state).unwrap().len() < 3 * 1024 * 1024);
        // Further review replaces a bounded check, never grows a lifetime history in the checkpoint.
        for n in 1..=100 {
            super::super::record(&mut state, event("M-K-001", sequence + n, true, false)).unwrap();
        }
        assert_eq!(state.practice["M-K-001"].checks.len(), 10);
        assert_eq!(state.practice["M-K-001"].explored.len(), 10);
    }

    #[test]
    fn repeated_difficulty_revisits_parent_then_returns_without_erasing_progress() {
        let mut state = MissionState::default();
        for n in 1..=10 {
            super::super::record(&mut state, event("M-K-001", n, true, false)).unwrap();
        }
        for n in 11..=12 {
            super::super::record(&mut state, event("M-K-003", n, false, false)).unwrap();
        }
        assert_eq!(
            select(&state, None, 0).unwrap().unwrap().skill_id,
            "M-K-001"
        );
        super::super::record(&mut state, event("M-K-001", 13, true, false)).unwrap();
        // M-K-003 also has another prerequisite; review all listed dependencies.
        let graph = CurriculumGraph::active().unwrap();
        for (i, id) in graph
            .get("M-K-003")
            .unwrap()
            .prerequisites
            .iter()
            .enumerate()
        {
            super::super::record(&mut state, event(id, 14 + i as u64, true, false)).unwrap();
        }
        assert_eq!(
            select(&state, None, 0).unwrap().unwrap().skill_id,
            "M-K-003"
        );
        assert!(completed("M-K-001", &state));
    }
}
