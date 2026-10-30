use super::*;
use std::{io::Write, net::TcpListener, thread};
use wonderwilds_director::story::{authored_scene, build_scene_input};
use wonderwilds_domain::story::State;

fn envelope(content: &Value) -> Value {
    json!({"choices":[{"finish_reason":"stop","message":{"content":content.to_string()}}]})
}

fn server(status: &str, body: String) -> (String, thread::JoinHandle<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/v1/chat/completions",
        listener.local_addr().unwrap()
    );
    let status = status.to_owned();
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut bytes = Vec::new();
        let (header_end, length) = loop {
            let mut chunk = [0; 4096];
            let n = socket.read(&mut chunk).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&chunk[..n]);
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                assert!(headers.contains("authorization: bearer synthetic.id"));
                let length: usize = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                break (end + 4, length);
            }
        };
        while bytes.len() < header_end + length {
            let mut chunk = [0; 4096];
            let n = socket.read(&mut chunk).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&chunk[..n]);
        }
        let request = serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap();
        write!(socket, "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{body}", body.len()).unwrap();
        request
    });
    (url, handle)
}

#[test]
fn sends_real_schema_and_caches_only_valid_scenes() {
    let state = State::new();
    let input = build_scene_input(&state).unwrap();
    let canonical = authored_scene(&state).unwrap();
    let (url, handle) = server("200 OK", envelope(&canonical).to_string());
    let provider = ModalSceneProvider::new(url, "synthetic".into(), "id".into()).unwrap();
    assert_eq!(provider.propose(&input, &canonical).unwrap(), canonical);
    let sent = handle.join().unwrap();
    assert_eq!(sent["model"], "wonderwilds");
    assert_eq!(sent["chat_template_kwargs"]["reasoning_strength"], "low");
    let contract: Value =
        serde_json::from_str(include_str!("../../../../../../contracts/schemas/v1.json")).unwrap();
    assert_eq!(
        sent["response_format"],
        json!({"type":"json_schema","json_schema":{
            "name":"story_scene","schema":contract["$defs"]["StoryScene"],"strict":true}})
    );
    let payload: Value =
        serde_json::from_str(sent["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(payload["input"], input);
    assert_eq!(payload["allowedSpeakerIds"], json!(["pip"]));
    assert!(payload["outputSchema"]["properties"]["choices"].is_object());
    assert_eq!(
        payload["outputSchema"],
        sent["response_format"]["json_schema"]["schema"]
    );
    // Server is gone: identical input is served from the validated cache.
    assert_eq!(provider.propose(&input, &canonical).unwrap(), canonical);
    let _busy = provider.cache.lock().unwrap();
    assert!(provider.propose(&input, &canonical).is_err());
}

#[test]
fn errors_oversize_truncation_and_invalid_bindings_are_rejected() {
    let state = State::new();
    let input = build_scene_input(&state).unwrap();
    let canonical = authored_scene(&state).unwrap();
    let mut invalid = canonical.clone();
    invalid["choices"] = json!([]);
    let mut invalid_binding = canonical.clone();
    invalid_binding["choices"][0]["actionId"] = json!("invented-action");
    // Strict schema output still needs authoritative gameplay validation.
    wonderwilds_contracts::validate("StoryScene", &invalid_binding).unwrap();
    for (status, body) in [
        ("400 Bad Request", json!({"error":{"message":"unsupported schema"}}).to_string()),
        ("503 Unavailable", "{}".into()),
        ("302 Found", "{}".into()),
        ("200 OK", "x".repeat(MAX_RESPONSE as usize + 1)),
        ("200 OK", "not JSON".into()),
        ("200 OK", envelope(&invalid).to_string()),
        ("200 OK", envelope(&invalid_binding).to_string()),
        ("200 OK", json!({"choices":[{"finish_reason":"length","message":{"content":canonical.to_string()}}]}).to_string()),
    ] {
        let (url, handle) = server(status, body);
        let provider = ModalSceneProvider::new(url, "synthetic".into(), "id".into()).unwrap();
        assert!(provider.propose(&input, &canonical).is_err());
        assert!(provider.cache.lock().unwrap().is_empty());
        handle.join().unwrap();
    }
    assert!(parse_response(&envelope(&json!("not an object"))).is_err());
}

#[test]
fn introduction_is_preserved_and_metadata_is_not_an_answer() {
    let mut state = State::new();
    state
        .apply(&wonderwilds_domain::story::Command {
            command_id: "travel".into(),
            expected_revision: 0,
            action: wonderwilds_domain::story::Action::Travel {
                destination_id: "workshop".into(),
            },
        })
        .unwrap();
    let input = build_scene_input(&state).unwrap();
    let canonical = authored_scene(&state).unwrap();
    let body = request_body(&input, &canonical);
    let payload: Value =
        serde_json::from_str(body["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        body["response_format"],
        json!({"type":"json_schema", "json_schema":{
        "name":"story_scene", "strict":true, "schema":payload["outputSchema"]}})
    );
    assert_eq!(
        payload["presentation"]["requiredEntries"][0]["entryId"],
        "greeting"
    );
    assert_eq!(payload["allowedSpeakerIds"], json!(["woodcutter"]));
    assert!(payload.get("reference").is_none());
}

#[test]
fn slow_endpoint_times_out_instead_of_blocking_play() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (_socket, _) = listener.accept().unwrap();
        thread::sleep(Duration::from_secs(7));
    });
    let provider = ModalSceneProvider::new(url, "synthetic".into(), "id".into()).unwrap();
    let started = std::time::Instant::now();
    assert!(provider.request(&json!({})).is_err());
    assert!(started.elapsed() < Duration::from_secs(7));
    server.join().unwrap();
}

#[test]
fn planner_sends_shared_dependency_prompt_and_closed_plan_schema() {
    let mut state = State::new();
    state
        .apply(&wonderwilds_domain::story::Command {
            command_id: "accept".into(),
            expected_revision: 0,
            action: wonderwilds_domain::story::Action::AcceptMission,
        })
        .unwrap();
    let input = wonderwilds_director::story::build_plan_input(&state).unwrap();
    let plan = wonderwilds_director::story::authored_plan(&state).unwrap();
    let (url, handle) = server("200 OK", envelope(&plan).to_string());
    let provider = ModalSceneProvider::new(url, "synthetic".into(), "id".into()).unwrap();
    let result = crate::story::planning::PlanProvider::propose(&provider, &input, &[]).unwrap();
    assert_eq!(result, plan);
    let sent = handle.join().unwrap();
    assert!(
        sent["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Follow prerequisites recursively")
    );
    let supplied: Value =
        serde_json::from_str(sent["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        sent["response_format"],
        response_format("StoryMissionPlan", &supplied["outputSchema"])
    );
    assert_eq!(supplied["input"], input);
    assert_eq!(supplied["completedConditionIds"], json!([]));
    assert!(supplied["outputSchema"]["$defs"]["StoryNarrative"].is_object());
    assert!(supplied.get("checkpoint").is_none());
    assert!(supplied.get("reference").is_none());
    let _busy = provider.cache.lock().unwrap();
    assert!(crate::story::planning::PlanProvider::propose(&provider, &input, &[]).is_err());
}

#[test]
fn exploration_requests_closed_schema_and_validates_model_created_characters_and_tasks() {
    let mut state = State::new();
    for destination in ["workshop", "cove"] {
        state
            .apply(&wonderwilds_domain::story::Command {
                command_id: destination.into(),
                expected_revision: state.revision(),
                action: wonderwilds_domain::story::Action::Travel {
                    destination_id: destination.into(),
                },
            })
            .unwrap();
    }
    let content = crate::story::generation::fallback(&state).unwrap();
    let (url, server) = server("200 OK", envelope(&content).to_string());
    let provider = ModalSceneProvider::new(url, "synthetic".into(), "id".into()).unwrap();
    let input = crate::story::generation::input(&state).unwrap();
    let output = crate::story::generation::ExplorationProvider::propose(&provider, &input).unwrap();
    assert_eq!(output, content);
    let body = server.join().unwrap();
    assert_eq!(
        body["response_format"]["json_schema"]["name"],
        "GeneratedNeighbourhood"
    );
    assert_eq!(body["response_format"]["json_schema"]["strict"], true);
    assert!(body["response_format"]["json_schema"]["schema"]["properties"]["steps"]["items"]["required"].as_array().unwrap().contains(&json!("learningBridge")));
    assert!(
        body["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("learningBridge")
    );
    let prompt = body["messages"][1]["content"].as_str().unwrap();
    assert!(!prompt.contains("profile"));
    assert!(!prompt.contains("session"));
    assert!(crate::story::planning::ValidatedPlan::generated(&state, output, true).is_ok());
    let mut bad = content;
    bad["steps"] = json!([]);
    assert!(crate::story::planning::ValidatedPlan::generated(&state, bad, true).is_err());
}

#[test]
fn credentials_select_generation_without_feature_flags() {
    assert!(
        ModalSceneProvider::with_config(None, None, None)
            .unwrap()
            .is_none()
    );
    assert!(
        ModalSceneProvider::with_config(Some(" ".into()), Some("".into()), None)
            .unwrap()
            .is_none()
    );
    for (id, secret) in [
        (Some("synthetic".into()), None),
        (None, Some("secret".into())),
    ] {
        assert!(ModalSceneProvider::with_config(id, secret, None).is_err());
    }
    let provider =
        ModalSceneProvider::with_config(Some("synthetic".into()), Some("id".into()), None)
            .unwrap()
            .unwrap();
    assert_eq!(provider.endpoint, ENDPOINT);
    assert_eq!(provider.authorization, "Bearer synthetic.id");
}

#[test]
fn shared_endpoint_override_is_restricted_to_https_or_loopback() {
    for endpoint in [
        "http://127.0.0.1:4321/v1/chat/completions",
        "https://example.com/v1/chat/completions",
    ] {
        let provider = ModalSceneProvider::with_config(
            Some("synthetic".into()),
            Some("id".into()),
            Some(endpoint.into()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(provider.endpoint, endpoint);
    }
    for endpoint in [
        "http://example.com/",
        "https://user:secret@example.com/",
        "https://example.com/?key=secret",
        "invalid",
    ] {
        assert!(
            ModalSceneProvider::with_config(
                Some("synthetic".into()),
                Some("id".into()),
                Some(endpoint.into())
            )
            .is_err()
        );
    }
}
