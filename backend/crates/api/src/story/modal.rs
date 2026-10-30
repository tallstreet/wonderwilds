//! Opt-in development-only scene prose and mission plans. No model credentials or identity reach the browser.
use serde_json::{Value, json};
use std::{collections::VecDeque, io::Read, sync::Mutex, time::Duration};
use wonderwilds_director::story::{Error, SceneProvider};

const ENDPOINT: &str =
    "https://garytafe--ep-wonderwilds-server.us-west.modal.direct/v1/chat/completions";
const MAX_RESPONSE: u64 = 64 * 1024;
const SYSTEM: &str = include_str!("modal_prompt.txt");

pub struct ModalSceneProvider {
    endpoint: String,
    authorization: String,
    // One outbound request at a time; busy callers immediately use authored prose.
    // Cache only bounded synthetic scene inputs, never identity/session information.
    cache: Mutex<VecDeque<(String, Value)>>,
}

fn unavailable() -> Error {
    Error::InvalidPayload("Scene provider unavailable".into())
}

impl ModalSceneProvider {
    pub fn from_env() -> Result<Option<Self>, &'static str> {
        Self::configured("WONDERWILDS_MODAL_STORY")
    }
    pub fn explorer_from_env() -> Result<Option<Self>, &'static str> {
        let flag = if std::env::var_os("WONDERWILDS_MODAL_EXPLORATION").is_some() {
            "WONDERWILDS_MODAL_EXPLORATION"
        } else {
            "WONDERWILDS_MODAL_STORY"
        };
        let mut provider = Self::configured(flag)?;
        if let Some(provider) = &mut provider
            && let Ok(endpoint) = std::env::var("WONDERWILDS_MODAL_EXPLORATION_ENDPOINT")
        {
            let url = reqwest::Url::parse(&endpoint).map_err(|_| "Invalid exploration endpoint")?;
            if !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
                || !(url.scheme() == "https"
                    || url.scheme() == "http"
                        && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost")))
            {
                return Err("Exploration endpoint requires HTTPS or loopback HTTP");
            }
            provider.endpoint = endpoint;
        }
        Ok(provider)
    }
    pub fn planner_from_env() -> Result<Option<Self>, &'static str> {
        Self::configured("WONDERWILDS_MODAL_PLANNER")
    }
    fn configured(flag: &str) -> Result<Option<Self>, &'static str> {
        match std::env::var(flag).as_deref() {
            Err(_) | Ok("0") => return Ok(None),
            Ok("1") => {}
            _ => return Err("Modal generation flags must be 0 or 1"),
        }
        let id =
            std::env::var("MODAL_PROXY_TOKEN_ID").map_err(|_| "Missing Modal proxy token ID")?;
        let secret = std::env::var("MODAL_PROXY_TOKEN_SECRET")
            .map_err(|_| "Missing Modal proxy token secret")?;
        Self::new(ENDPOINT.into(), id, secret).map(Some)
    }

    fn new(endpoint: String, id: String, secret: String) -> Result<Self, &'static str> {
        if id.is_empty() || secret.is_empty() {
            return Err("Modal credentials must be nonempty");
        }
        let authorization = format!("Bearer {id}.{secret}");
        reqwest::header::HeaderValue::from_str(&authorization)
            .map_err(|_| "Invalid Modal credentials")?;
        Ok(Self {
            endpoint,
            authorization,
            cache: Mutex::new(VecDeque::new()),
        })
    }

    fn request(&self, body: &Value) -> Result<Value, Error> {
        // Construct/use/drop the blocking client only inside the HTTP blocking worker.
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(6))
            .build()
            .map_err(|_| unavailable())?;
        let response = client
            .post(&self.endpoint)
            .header(reqwest::header::AUTHORIZATION, &self.authorization)
            .json(body)
            .send()
            .map_err(|_| unavailable())?;
        if !response.status().is_success() {
            return Err(unavailable());
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| unavailable())?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(unavailable());
        }
        parse_response(&serde_json::from_slice(&bytes).map_err(|_| unavailable())?)
    }
}

fn request_body(input: &Value, canonical: &Value) -> Value {
    let schema: Value =
        serde_json::from_str(include_str!("../../../../../contracts/schemas/v1.json"))
            .expect("checked-in contract schema");
    // Required introduction is authoritative. Other authored prose is not sent as an answer.
    let required: Vec<_> = canonical["presentation"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["entryId"] == "greeting")
        .cloned()
        .collect();
    let allowed: Vec<_> = input["here"]["characters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|character| character["id"].clone())
        .collect();
    let supplied = json!({"input":input,
        "presentation":{"sceneId":canonical["sceneId"],"requiredEntries":required},
        "allowedSpeakerIds":allowed,"outputSchema":schema["$defs"]["StoryScene"],
        "editorialBrief":"Ages 6–8; short, warm sentences; concrete next steps; no frightening or manipulative content."});
    json!({"model":"wonderwilds","messages":[
        {"role":"system","content":SYSTEM},
        {"role":"user","content":supplied.to_string()}],
        "response_format":response_format("story_scene", &schema["$defs"]["StoryScene"]),
        "temperature":0,"max_tokens":2048,"stream":false,
        "chat_template_kwargs":{"reasoning_strength":"low"}})
}

fn response_format(name: &str, schema: &Value) -> Value {
    json!({"type":"json_schema","json_schema":{"name":name,"strict":true,"schema":schema}})
}

fn parse_response(envelope: &Value) -> Result<Value, Error> {
    if envelope.get("error").is_some_and(|error| !error.is_null())
        || envelope["choices"][0]["finish_reason"] != "stop"
    {
        return Err(unavailable());
    }
    let content = envelope["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(unavailable)?;
    // OpenAI-compatible serving separates reasoning; accept final JSON only, no repair.
    let value: Value = serde_json::from_str(content).map_err(|_| unavailable())?;
    if !value.is_object() {
        return Err(unavailable());
    }
    Ok(value)
}

impl SceneProvider for ModalSceneProvider {
    fn propose(&self, input: &Value, canonical: &Value) -> Result<Value, Error> {
        wonderwilds_contracts::validate("StorySceneInput", input).map_err(|_| unavailable())?;
        wonderwilds_contracts::validate("StoryScene", canonical).map_err(|_| unavailable())?;
        let body = request_body(input, canonical);
        let key = body.to_string();
        let mut cache = self.cache.try_lock().map_err(|_| unavailable())?;
        if let Some((_, scene)) = cache.iter().find(|(stored, _)| stored == &key) {
            return Ok(scene.clone());
        }
        let proposal = self.request(&body)?;
        wonderwilds_director::story::validate_scene(input, canonical, &proposal)?;
        if cache.len() >= 32 {
            cache.pop_front();
        }
        cache.push_back((key, proposal.clone()));
        Ok(proposal)
    }
}

#[cfg(test)]
mod tests;

impl super::planning::PlanProvider for ModalSceneProvider {
    fn propose(&self, input: &Value, completed: &[String]) -> Result<Value, Error> {
        wonderwilds_contracts::validate("StoryPlanInput", input).map_err(|_| unavailable())?;
        let schema: Value =
            serde_json::from_str(include_str!("../../../../../contracts/schemas/v1.json")).unwrap();
        let mut output_schema = schema["$defs"]["StoryMissionPlan"].clone();
        output_schema["$defs"] = json!({"StoryNarrative": schema["$defs"]["StoryNarrative"]});
        let supplied =
            json!({"input":input,"completedConditionIds":completed,"outputSchema":output_schema});
        let system = format!(
            "{}{}",
            include_str!("../../../director/src/story/prompts/mission_baseline.txt"),
            include_str!("../../../director/src/story/prompts/mission_dependencies.txt")
        );
        let body = json!({"model":"wonderwilds","messages":[{"role":"system","content":system},
            {"role":"user","content":supplied.to_string()}], "temperature":0,"max_tokens":2048,
            "stream":false,"response_format":response_format("StoryMissionPlan", &output_schema),
            "chat_template_kwargs":{"reasoning_strength":"low"}});
        // A busy planner falls back immediately. Accepted plans are persisted by
        // the repository, not cached across player state or command identities.
        let _guard = self.cache.try_lock().map_err(|_| unavailable())?;
        let plan = self.request(&body)?;
        wonderwilds_contracts::validate("StoryMissionPlan", &plan).map_err(|_| unavailable())?;
        Ok(plan)
    }
}

impl super::generation::ExplorationProvider for ModalSceneProvider {
    fn propose(&self, input: &Value) -> Result<Value, Error> {
        let schema: Value =
            serde_json::from_str(include_str!("../../../../../contracts/schemas/v1.json")).unwrap();
        // New generations must supply the join. The storage/wire schema keeps
        // it optional so historical content and offline fallbacks still load.
        let mut output_schema = schema["$defs"]["GeneratedNeighbourhood"].clone();
        output_schema["properties"]["steps"]["items"]["required"]
            .as_array_mut()
            .expect("step required fields")
            .push(json!("learningBridge"));
        let body = json!({"model":"wonderwilds","messages":[
            {"role":"system","content":include_str!("exploration_prompt.txt")},
            {"role":"user","content":input.to_string()}],
            "response_format":response_format("GeneratedNeighbourhood", &output_schema),
            "temperature":0.8,"max_tokens":1600,"stream":false,
            "chat_template_kwargs":{"reasoning_strength":"low"}});
        let _guard = self.cache.try_lock().map_err(|_| unavailable())?;
        let proposal = self.request(&body)?;
        wonderwilds_contracts::validate("GeneratedNeighbourhood", &proposal)
            .map_err(|_| unavailable())?;
        Ok(proposal)
    }
}
