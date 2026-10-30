use super::super::repository::MemoryRepository;
use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use std::sync::Mutex;
use tower::ServiceExt;
struct SyntheticAuth;
impl Authenticator for SyntheticAuth {
    fn authenticate(&self, headers: &HeaderMap, write: bool, _: u64) -> Result<Identity, Error> {
        let parent = match headers.get("cookie").and_then(|v| v.to_str().ok()) {
            Some("test-parent-a") => "parent-a",
            Some("test-parent-b") => "parent-b",
            _ => return Err(Error::Unauthenticated),
        };
        if write
            && (headers.get("x-csrf-token").and_then(|v| v.to_str().ok()) != Some("synthetic-csrf")
                || headers.get("origin").and_then(|v| v.to_str().ok())
                    != Some("https://game.example"))
        {
            return Err(Error::Forbidden);
        }
        let profile = headers
            .get("x-wonderwilds-profile")
            .and_then(|v| v.to_str().ok())
            .ok_or(Error::Forbidden)?;
        Ok(Identity {
            parent_id: parent.into(),
            profile_id: profile.into(),
            session_id: format!("session-{parent}"),
        })
    }
}
fn app() -> (Router, Arc<MemoryRepository>) {
    let content = development_content().unwrap();
    let repo = Arc::new(MemoryRepository {
        content: content.clone(),
        profiles: Mutex::new(BTreeMap::from([
            (
                "profile-a".into(),
                ("parent-a".into(), Aggregate::new(&content)),
            ),
            (
                "profile-b".into(),
                ("parent-b".into(), Aggregate::new(&content)),
            ),
        ])),
    });
    let state = AppState {
        repository: repo.clone(),
        auth: Arc::new(SyntheticAuth),
        story_repository: Arc::new(StoryMemory::new()),
        story_provider: None,
        story_planner: None,
        content,
        permits: Arc::new(tokio::sync::Semaphore::new(16)),
        configured: true,
    };
    (router(state), repo)
}
async fn send(
    app: Router,
    path: &str,
    value: Option<Value>,
    parent: &str,
    profile: &str,
    csrf: bool,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .uri(path)
        .header("cookie", parent)
        .header("x-wonderwilds-profile", profile)
        .header("origin", "https://game.example");
    if csrf {
        req = req.header("x-csrf-token", "synthetic-csrf");
    }
    let req = if let Some(value) = value {
        req.method("POST")
            .header("content-type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap()
    } else {
        req.body(Body::empty()).unwrap()
    };
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 32768).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}
fn cmd(id: &str, revision: u64, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":revision,"payload":payload})
}
#[tokio::test]
async fn missing_identity_csrf_and_cross_profile_fail_closed() {
    let (app, _) = app();
    let (status, body) = send(
        app.clone(),
        "/api/v1/game",
        None,
        "none",
        "profile-a",
        false,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "UNAUTHENTICATED");
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/game",
            None,
            "test-parent-b",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let request = cmd(
        "start",
        0,
        json!({"type":"start_encounter","beatId":"meadow-welcome"}),
    );
    assert_eq!(
        send(
            app,
            "/api/v1/commands",
            Some(request),
            "test-parent-a",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
async fn http_contract_stale_retry_body_tampering_and_route_mismatch() {
    let (app, _) = app();
    let start = cmd(
        "start",
        0,
        json!({"type":"start_encounter","beatId":"meadow-welcome"}),
    );
    let (status, snapshot) = send(
        app.clone(),
        "/api/v1/commands",
        Some(start.clone()),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    wire::validate("GameSnapshot", &snapshot).unwrap();
    assert_eq!(snapshot["revision"], 1);
    let eid = snapshot["pendingEncounters"][0]["id"].as_str().unwrap();
    assert!(snapshot["pendingEncounters"][0].get("answer").is_none());
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/commands",
            Some(start),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .1,
        snapshot
    );
    let mut forged = cmd(
        "start",
        0,
        json!({"type":"start_encounter","beatId":"meadow-bridge"}),
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/commands",
            Some(forged.clone()),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    forged["commandId"] = json!("different");
    forged["parentId"] = json!("parent-b");
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/commands",
            Some(forged),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let hint = cmd(
        "hint",
        1,
        json!({"type":"hint","encounterId":eid,"supportKind":"hint"}),
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/encounters/not-owned/hints",
            Some(hint.clone()),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        send(
            app.clone(),
            &format!("/api/v1/encounters/{eid}"),
            None,
            "test-parent-b",
            "profile-b",
            false
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/commands",
            Some(hint),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::OK
    );
    let stale = cmd(
        "stale",
        1,
        json!({"type":"hint","encounterId":eid,"supportKind":"hint"}),
    );
    assert_eq!(
        send(
            app,
            "/api/v1/commands",
            Some(stale),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn concurrent_duplicate_answers_have_one_reward_and_preserve_support() {
    let (app, repo) = app();
    let (_, snapshot) = send(
        app.clone(),
        "/api/v1/commands",
        Some(cmd(
            "start",
            0,
            json!({"type":"start_encounter","beatId":"meadow-welcome"}),
        )),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    let eid = snapshot["pendingEncounters"][0]["id"].as_str().unwrap();
    let (_, hinted) = send(
        app.clone(),
        "/api/v1/commands",
        Some(cmd(
            "hint",
            1,
            json!({"type":"hint","encounterId":eid,"supportKind":"worked_example"}),
        )),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    assert_eq!(hinted["pendingEncounters"][0]["skillId"], "E01");
    assert_eq!(hinted["learnerSummaries"][0]["status"], "introduced");
    let request = cmd(
        "answer",
        2,
        json!({"type":"attempt","encounterId":eid,"answer":{"kind":"choices","choiceIds":["message-0"]}}),
    );
    let (a, b) = tokio::join!(
        send(
            app.clone(),
            "/api/v1/commands",
            Some(request.clone()),
            "test-parent-a",
            "profile-a",
            true
        ),
        send(
            app.clone(),
            "/api/v1/commands",
            Some(request),
            "test-parent-a",
            "profile-a",
            true
        )
    );
    assert_eq!(a.0, StatusCode::OK);
    assert_eq!(a, b);
    assert_eq!(a.1["learnerSummaries"][0]["status"], "practising");
    assert!(
        a.1["learnerSummaries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["status"] != "secure")
    );
    let profiles = repo.profiles.lock().unwrap();
    let aggregate = &profiles["profile-a"].1;
    assert_eq!(aggregate.evidence.len(), 3);
    assert!(aggregate.evidence[1].support.worked_example);
    assert_eq!(
        aggregate
            .events
            .iter()
            .filter(|e| matches!(e.outcome, engine::Outcome::QuestCompleted { .. }))
            .count(),
        1
    );
    assert!(aggregate.domain.completed().contains("meadow-welcome"));
    let grants = aggregate
        .events
        .iter()
        .filter(|e| matches!(e.outcome, engine::Outcome::RewardGranted { .. }))
        .count();
    assert!(grants > 0);
}
#[tokio::test]
async fn unconfigured_provider_has_health_but_never_issues_sessions() {
    let state = AppState::development(None, "https://game.example".into()).unwrap();
    let app = router(state);
    assert_eq!(
        send(app.clone(), "/healthz", None, "none", "none", false)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        send(app.clone(), "/readyz", None, "none", "none", false)
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        send(
            app,
            "/api/v1/game",
            None,
            "test-parent-a",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn configured_but_unavailable_database_is_not_ready() {
    let state=AppState::development(Some("host=/tmp/wonderwilds-no-such-socket port=55499 user=unavailable dbname=unavailable connect_timeout=1".into()),"https://game.example".into()).unwrap();
    assert_eq!(
        send(router(state), "/readyz", None, "none", "none", false)
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(matches!(
        AppState::development(
            Some("host=/tmp host=external.example user=unavailable".into()),
            "https://game.example".into()
        ),
        Err(Error::InvalidInput)
    ));
}

#[tokio::test]
async fn abandonment_releases_pending_slot_without_rewards_and_is_idempotent() {
    async fn submit(app: &Router, value: Value) -> (StatusCode, Value) {
        send(
            app.clone(),
            "/api/v1/commands",
            Some(value),
            "test-parent-a",
            "profile-a",
            true,
        )
        .await
    }
    let (app, repo) = app();
    let start = cmd(
        "abandon-start",
        0,
        json!({"type":"start_encounter","beatId":"meadow-welcome"}),
    );
    let (status, started) = submit(&app, start).await;
    assert_eq!(status, StatusCode::OK);
    let encounter = started["pendingEncounters"][0]["id"].as_str().unwrap();
    let abandon = cmd(
        "abandon-command",
        1,
        json!({"type":"abandon","encounterId":encounter}),
    );
    // A foreign profile cannot close someone else's encounter.
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/commands",
            Some(cmd(
                "foreign-abandon",
                0,
                json!({"type":"abandon","encounterId":encounter})
            )),
            "test-parent-b",
            "profile-b",
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (status, ended) = submit(&app, abandon.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ended["revision"], 2);
    assert_eq!(ended["pendingEncounters"], json!([]));
    for field in ["storyFlags", "ownedCreatures", "inventory"] {
        assert_eq!(ended[field], started[field]);
    }
    assert_eq!(ended["learnerSummaries"][0]["status"], "introduced");
    assert_eq!(
        submit(&app, abandon.clone()).await,
        (StatusCode::OK, ended.clone())
    );
    {
        let profiles = repo.profiles.lock().unwrap();
        let aggregate = &profiles["profile-a"].1;
        assert_eq!(aggregate.evidence.len(), 1);
        assert_eq!(
            aggregate.evidence[0].event_kind,
            wire::LearningEvidenceEventKind::Abandonment
        );
        assert_eq!(
            aggregate.evidence[0].result,
            wire::LearningEvidenceResult::NotAssessed
        );
        assert!(!aggregate.evidence[0].first_check);
        assert!(aggregate.domain.completed().is_empty());
    }
    assert_eq!(
        send(
            app.clone(),
            &format!("/api/v1/encounters/{encounter}"),
            None,
            "test-parent-a",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let attempt = cmd(
        "answer-after-abandon",
        2,
        json!({"type":"attempt","encounterId":encounter,"answer":{"kind":"choices","choiceIds":["message-0"]}}),
    );
    assert_eq!(submit(&app, attempt).await.0, StatusCode::CONFLICT);
    let restart = cmd(
        "abandon-restart",
        2,
        json!({"type":"start_encounter","beatId":"meadow-welcome"}),
    );
    let (status, restarted) = submit(&app, restart).await;
    assert_eq!(status, StatusCode::OK);
    let next_encounter = restarted["pendingEncounters"][0]["id"].as_str().unwrap();
    assert_ne!(next_encounter, encounter);
    assert_eq!(restarted["revision"], 3);
    // Retrying an old cancellation does not cancel the newly issued encounter.
    assert_eq!(submit(&app, abandon).await, (StatusCode::OK, ended));
    assert_eq!(
        repo.profiles.lock().unwrap()["profile-a"]
            .1
            .public_encounter(next_encounter)
            .unwrap()["id"],
        next_encounter
    );
}

#[test]
fn saved_encounters_without_abandonment_field_remain_readable() {
    let content = development_content().unwrap();
    let mut aggregate = Aggregate::new(&content);
    aggregate
        .execute(
            &cmd(
                "legacy-start",
                0,
                json!({"type":"start_encounter","beatId":"meadow-welcome"}),
            ),
            &content,
            1,
            "session-a",
        )
        .unwrap();
    let mut legacy = serde_json::to_value(&aggregate).unwrap();
    for encounter in legacy["encounters"].as_object_mut().unwrap().values_mut() {
        encounter.as_object_mut().unwrap().remove("abandoned");
    }
    let restored: Aggregate = serde_json::from_value(legacy).unwrap();
    assert_eq!(
        restored.snapshot(&content).unwrap(),
        aggregate.snapshot(&content).unwrap()
    );
}

struct StoryMemory {
    profiles: Mutex<BTreeMap<String, StoryRecord>>,
}
struct StoryRecord {
    owner: String,
    state: wonderwilds_domain::story::State,
    receipts: BTreeMap<String, (Value, Value)>,
    plans: BTreeMap<String, Value>,
}
impl StoryMemory {
    fn new() -> Self {
        Self {
            profiles: Mutex::new(BTreeMap::from([(
                "profile-a".into(),
                StoryRecord {
                    owner: "parent-a".into(),
                    state: wonderwilds_domain::story::State::new(),
                    receipts: BTreeMap::new(),
                    plans: BTreeMap::new(),
                },
            )])),
        }
    }
}
impl crate::story::repository::Repository for StoryMemory {
    fn read(&self, identity: &Identity) -> Result<Value, Error> {
        let profiles = self.profiles.lock().map_err(|_| Error::Unavailable)?;
        let record = profiles
            .get(&identity.profile_id)
            .filter(|r| r.owner == identity.parent_id)
            .ok_or(Error::Forbidden)?;
        let mut snapshot = crate::story::snapshot(&record.state)?;
        if let Some(plan) = record.plans.get(record.state.mission_id()) {
            crate::story::planning::project(&record.state, plan, &mut snapshot)?;
        }
        Ok(snapshot)
    }
    fn planning_state(
        &self,
        identity: &Identity,
        request: &Value,
    ) -> Result<Option<wonderwilds_domain::story::State>, Error> {
        let profiles = self.profiles.lock().unwrap();
        let record = profiles
            .get(&identity.profile_id)
            .filter(|r| r.owner == identity.parent_id)
            .ok_or(Error::Forbidden)?;
        if record
            .receipts
            .contains_key(request["commandId"].as_str().unwrap_or(""))
        {
            return Ok(None);
        }
        crate::story::planning::acceptance(&record.state, request)
    }
    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error> {
        self.execute_planned(identity, request, None)
    }
    fn execute_planned(
        &self,
        identity: &Identity,
        request: &Value,
        plan: Option<&crate::story::planning::ValidatedPlan>,
    ) -> Result<Value, Error> {
        wire::validate("StoryCommand", request).map_err(|_| Error::InvalidInput)?;
        let mut profiles = self.profiles.lock().map_err(|_| Error::Unavailable)?;
        let record = profiles
            .get_mut(&identity.profile_id)
            .filter(|r| r.owner == identity.parent_id)
            .ok_or(Error::Forbidden)?;
        let id = request["commandId"].as_str().unwrap();
        if let Some((previous, response)) = record.receipts.get(id) {
            return if previous == request {
                Ok(response.clone())
            } else {
                Err(Error::RevisionConflict)
            };
        }
        let (next, mut response) = crate::story::execute(&record.state, request)?;
        if let Some(plan) = plan {
            record
                .plans
                .insert(next.mission_id().into(), plan.for_state(&next)?.clone());
        }
        if let Some(plan) = record.plans.get(next.mission_id()) {
            crate::story::planning::project(&next, plan, &mut response)?;
        }
        record.state = next;
        record
            .receipts
            .insert(id.into(), (request.clone(), response.clone()));
        Ok(response)
    }
}

#[tokio::test]
async fn story_routes_share_auth_csrf_ownership_and_closed_command_boundary() {
    let (app, _) = app();
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story",
            None,
            "none",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story",
            None,
            "test-parent-b",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, initial) = send(
        app.clone(),
        "/api/v1/story",
        None,
        "test-parent-a",
        "profile-a",
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    wire::validate("StorySnapshot", &initial).unwrap();
    let accept = cmd(
        "story-accept",
        0,
        json!({"type":"choose","actionId":initial["sceneInput"]["availableActions"][0]["id"]}),
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story/commands",
            Some(accept.clone()),
            "test-parent-a",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let untrusted_origin = Request::builder()
        .uri("/api/v1/story/commands")
        .method("POST")
        .header("cookie", "test-parent-a")
        .header("x-wonderwilds-profile", "profile-a")
        .header("x-csrf-token", "synthetic-csrf")
        .header("origin", "https://untrusted.example")
        .header("content-type", "application/json")
        .body(Body::from(accept.to_string()))
        .unwrap();
    assert_eq!(
        app.clone()
            .oneshot(untrusted_origin)
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    let accepted = send(
        app.clone(),
        "/api/v1/story/commands",
        Some(accept.clone()),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    assert_eq!(accepted.0, StatusCode::OK);
    assert_eq!(accepted.1["revision"], 1);
    let travel = accepted.1["sceneInput"]["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"]["type"] == "travel")
        .unwrap();
    let request = cmd(
        "story-travel",
        1,
        json!({"type":"choose","actionId":travel["id"]}),
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story/commands",
            Some(request),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::OK
    );
    // Retry returns the original revision-one response, not the current snapshot.
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story/commands",
            Some(accept.clone()),
            "test-parent-a",
            "profile-a",
            true
        )
        .await,
        accepted
    );
    let mut wrong = accept.clone();
    wrong["payload"]["actionId"] = json!("invented");
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story/commands",
            Some(wrong),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut wrong = accept.clone();
    wrong["commandId"] = json!("stale-new-id");
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story/commands",
            Some(wrong),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut wrong = accept;
    wrong["profileId"] = json!("profile-b");
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story/commands",
            Some(wrong),
            "test-parent-a",
            "profile-a",
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_, game) = send(
        app,
        "/api/v1/game",
        None,
        "test-parent-a",
        "profile-a",
        false,
    )
    .await;
    assert_eq!(
        game["revision"], 0,
        "Story writes changed assessed-game progress"
    );
}

struct TestSceneProvider {
    calls: Arc<std::sync::atomic::AtomicUsize>,
    behavior: u8,
    repository: Arc<StoryMemory>,
}
impl wonderwilds_director::story::SceneProvider for TestSceneProvider {
    fn propose(
        &self,
        _: &Value,
        canonical: &Value,
    ) -> Result<Value, wonderwilds_director::story::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.behavior == 2 {
            return Err(wonderwilds_director::story::Error::InvalidPayload(
                "offline".into(),
            ));
        }
        if self.behavior == 3 {
            let mut profiles = self.repository.profiles.lock().unwrap();
            let record = profiles.get_mut("profile-a").unwrap();
            record
                .state
                .apply(&wonderwilds_domain::story::Command {
                    command_id: "concurrent-travel".into(),
                    expected_revision: record.state.revision(),
                    action: wonderwilds_domain::story::Action::Travel {
                        destination_id: "workshop".into(),
                    },
                })
                .unwrap();
        }
        let mut proposed = canonical.clone();
        proposed["presentation"][0]["text"] = json!("A generated scene.");
        if self.behavior == 1 {
            proposed["choices"] = json!([]);
        }
        Ok(proposed)
    }
}

fn model_app(behavior: u8) -> (Router, Arc<std::sync::atomic::AtomicUsize>) {
    let repository = Arc::new(StoryMemory::new());
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut state = AppState::development(None, "https://game.example".into()).unwrap();
    state.auth = Arc::new(SyntheticAuth);
    state.story_repository = repository.clone();
    state.story_provider = Some(Arc::new(TestSceneProvider {
        calls: calls.clone(),
        behavior,
        repository,
    }));
    (router(state), calls)
}

#[tokio::test]
async fn generated_reads_require_auth_and_preserve_command_receipts() {
    let (app, calls) = model_app(0);
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story",
            None,
            "none",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(
            app.clone(),
            "/api/v1/story",
            None,
            "test-parent-b",
            "profile-a",
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let (status, scene) = send(
        app.clone(),
        "/api/v1/story",
        None,
        "test-parent-a",
        "profile-a",
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        scene["scene"]["presentation"][0]["text"],
        "A generated scene."
    );
    let command = cmd(
        "model-choice",
        0,
        json!({"type":"choose","actionId":scene["scene"]["choices"][0]["actionId"]}),
    );
    let first = send(
        app.clone(),
        "/api/v1/story/commands",
        Some(command.clone()),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    let retry = send(
        app,
        "/api/v1/story/commands",
        Some(command),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first, retry);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invalid_failed_and_stale_generations_fall_back_to_current_authored_scene() {
    for behavior in [1, 2, 3] {
        let (app, _) = model_app(behavior);
        let (status, scene) = send(
            app,
            "/api/v1/story",
            None,
            "test-parent-a",
            "profile-a",
            false,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_ne!(
            scene["scene"]["presentation"][0]["text"],
            "A generated scene."
        );
        assert_eq!(scene["revision"], if behavior == 3 { 1 } else { 0 });
        wire::validate("StorySnapshot", &scene).unwrap();
    }
}

struct ExpiringSceneAuth(std::sync::atomic::AtomicUsize);
impl Authenticator for ExpiringSceneAuth {
    fn authenticate(&self, headers: &HeaderMap, write: bool, now: u64) -> Result<Identity, Error> {
        if self.0.fetch_add(1, Ordering::SeqCst) > 0 {
            return Err(Error::Unauthenticated);
        }
        SyntheticAuth.authenticate(headers, write, now)
    }
}

#[tokio::test]
async fn expired_session_during_generation_does_not_receive_model_output() {
    let repository = Arc::new(StoryMemory::new());
    let mut state = AppState::development(None, "https://game.example".into()).unwrap();
    state.auth = Arc::new(ExpiringSceneAuth(std::sync::atomic::AtomicUsize::new(0)));
    state.story_repository = repository.clone();
    state.story_provider = Some(Arc::new(TestSceneProvider {
        calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        behavior: 0,
        repository,
    }));
    let (status, _) = send(
        router(state),
        "/api/v1/story",
        None,
        "test-parent-a",
        "profile-a",
        false,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

struct TestPlanProvider {
    calls: Arc<std::sync::atomic::AtomicUsize>,
    behavior: u8,
    repository: Arc<StoryMemory>,
}
impl crate::story::planning::PlanProvider for TestPlanProvider {
    fn propose(
        &self,
        _: &Value,
        _: &[String],
    ) -> Result<Value, wonderwilds_director::story::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.behavior == 1 {
            return Err(wonderwilds_director::story::Error::InvalidPayload(
                "offline".into(),
            ));
        }
        if self.behavior == 3 {
            let mut profiles = self.repository.profiles.lock().unwrap();
            profiles
                .get_mut("profile-a")
                .unwrap()
                .state
                .apply(&wonderwilds_domain::story::Command {
                    command_id: "concurrent".into(),
                    expected_revision: 0,
                    action: wonderwilds_domain::story::Action::Travel {
                        destination_id: "workshop".into(),
                    },
                })
                .unwrap();
        }
        let mut plan =
            wonderwilds_director::story::authored_plan(&wonderwilds_domain::story::State::new())
                .unwrap();
        plan["goal"] = json!("A planned crossing with Pip");
        plan["milestones"][0]["label"] = json!("Find wood for our crossing.");
        if self.behavior == 2 {
            plan["milestones"].as_array_mut().unwrap().swap(0, 1);
        }
        Ok(plan)
    }
}

#[tokio::test]
async fn planner_acceptance_persists_retries_falls_back_and_rejects_races() {
    for behavior in 0..4 {
        let repository = Arc::new(StoryMemory::new());
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut state = AppState::development(None, "https://game.example".into()).unwrap();
        state.auth = Arc::new(SyntheticAuth);
        state.story_repository = repository.clone();
        state.story_planner = Some(Arc::new(TestPlanProvider {
            calls: calls.clone(),
            behavior,
            repository: repository.clone(),
        }));
        let app = router(state);
        let (_, initial) = send(
            app.clone(),
            "/api/v1/story",
            None,
            "test-parent-a",
            "profile-a",
            false,
        )
        .await;
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let accept = cmd(
            "planned-accept",
            0,
            json!({"type":"choose","actionId":initial["sceneInput"]["availableActions"][0]["id"]}),
        );
        assert_eq!(
            send(
                app.clone(),
                "/api/v1/story/commands",
                Some(accept.clone()),
                "test-parent-a",
                "profile-a",
                false
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            send(
                app.clone(),
                "/api/v1/story/commands",
                Some(accept.clone()),
                "test-parent-b",
                "profile-a",
                true
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let response = send(
            app.clone(),
            "/api/v1/story/commands",
            Some(accept.clone()),
            "test-parent-a",
            "profile-a",
            true,
        )
        .await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        if behavior == 3 {
            assert_eq!(response.0, StatusCode::CONFLICT);
            continue;
        }
        assert_eq!(response.0, StatusCode::OK);
        assert_eq!(
            response.1["sceneInput"]["mission"]["goal"] == "A planned crossing with Pip",
            behavior == 0
        );
        assert_eq!(
            send(
                app.clone(),
                "/api/v1/story",
                None,
                "test-parent-a",
                "profile-a",
                false
            )
            .await
            .1,
            response.1
        );
        assert_eq!(
            send(
                app.clone(),
                "/api/v1/story/commands",
                Some(accept),
                "test-parent-a",
                "profile-a",
                true
            )
            .await,
            response
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let cancel = response.1["sceneInput"]["availableActions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["action"]["type"] == "cancel_mission")
            .unwrap();
        let (_, paused) = send(
            app.clone(),
            "/api/v1/story/commands",
            Some(cmd(
                "pause",
                1,
                json!({"type":"choose","actionId":cancel["id"]}),
            )),
            "test-parent-a",
            "profile-a",
            true,
        )
        .await;
        let resume = paused["sceneInput"]["availableActions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["action"]["type"] == "accept_mission")
            .unwrap();
        let (_, resumed) = send(
            app,
            "/api/v1/story/commands",
            Some(cmd(
                "resume",
                2,
                json!({"type":"choose","actionId":resume["id"]}),
            )),
            "test-parent-a",
            "profile-a",
            true,
        )
        .await;
        assert_eq!(
            resumed["sceneInput"]["mission"]["goal"],
            response.1["sceneInput"]["mission"]["goal"]
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn planner_rechecks_expired_session_before_committing_acceptance() {
    let repository = Arc::new(StoryMemory::new());
    let initial = crate::story::snapshot(&wonderwilds_domain::story::State::new()).unwrap();
    let mut state = AppState::development(None, "https://game.example".into()).unwrap();
    state.auth = Arc::new(ExpiringSceneAuth(std::sync::atomic::AtomicUsize::new(0)));
    state.story_repository = repository.clone();
    state.story_planner = Some(Arc::new(TestPlanProvider {
        calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        behavior: 0,
        repository: repository.clone(),
    }));
    let request = cmd(
        "expired-plan",
        0,
        json!({"type":"choose","actionId":initial["sceneInput"]["availableActions"][0]["id"]}),
    );
    let (status, _) = send(
        router(state),
        "/api/v1/story/commands",
        Some(request),
        "test-parent-a",
        "profile-a",
        true,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let records = repository.profiles.lock().unwrap();
    assert_eq!(records["profile-a"].state.revision(), 0);
    assert!(records["profile-a"].plans.is_empty());
    assert!(records["profile-a"].receipts.is_empty());
}
