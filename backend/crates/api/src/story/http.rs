//! Story routes reuse the same authenticated profile/CSRF boundary as gameplay.
use crate::game::{Error, http::AppState};
use axum::{Json, extract::State as App, http::HeaderMap};
use serde_json::Value;

type Body = Result<Json<Value>, axum::extract::rejection::JsonRejection>;

async fn run(
    state: AppState,
    headers: HeaderMap,
    request: Option<Value>,
) -> Result<Json<Value>, Error> {
    let permit = state
        .permits
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::RateLimited)?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::Unavailable)?
            .as_secs();
        let identity = state.auth.authenticate(&headers, request.is_some(), now)?;
        match request {
            Some(request) => {
                let mut plan = None;
                if let Some(provider) = &state.story_planner
                    && let Some(candidate) =
                        state.story_repository.planning_state(&identity, &request)?
                {
                    let input = wonderwilds_director::story::build_plan_input(&candidate)
                        .map_err(|_| Error::Unavailable)?;
                    let completed = candidate
                        .milestones()
                        .iter()
                        .filter(|m| m.complete)
                        .map(|m| m.condition_id.clone())
                        .collect::<Vec<_>>();
                    match provider.propose(&input, &completed) {
                        Ok(value) => {
                            plan = super::planning::ValidatedPlan::new(&candidate, value).ok();
                            eprintln!(
                                "Modal planner: {}",
                                if plan.is_some() {
                                    "validated plan"
                                } else {
                                    "validation failed; authored fallback"
                                }
                            );
                        }
                        Err(_) => eprintln!(
                            "Modal planner: provider unavailable or malformed; authored fallback"
                        ),
                    }
                    // Provider and bounded search run outside DB locks. Recheck the
                    // session and CSRF before committing; repository rechecks revision.
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|_| Error::Unavailable)?
                        .as_secs();
                    let fresh = state.auth.authenticate(&headers, true, now)?;
                    if fresh.parent_id != identity.parent_id
                        || fresh.profile_id != identity.profile_id
                        || fresh.session_id != identity.session_id
                    {
                        return Err(Error::Forbidden);
                    }
                }
                state
                    .story_repository
                    .execute_planned(&identity, &request, plan.as_ref())
                    .map(Json)
            }
            None => {
                let mut snapshot = state.story_repository.read(&identity)?;
                if let Some(provider) = &state.story_provider
                    && snapshot["commandBudgetExhausted"] == false
                {
                    let proposal = provider.propose(&snapshot["sceneInput"], &snapshot["scene"]);
                    // Recheck authentication and current state after potentially slow I/O.
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|_| Error::Unavailable)?
                        .as_secs();
                    let current_identity = state.auth.authenticate(&headers, false, now)?;
                    if current_identity.parent_id != identity.parent_id
                        || current_identity.profile_id != identity.profile_id
                        || current_identity.session_id != identity.session_id
                    {
                        return Err(Error::Forbidden);
                    }
                    let mut fresh = state.story_repository.read(&current_identity)?;
                    if fresh != snapshot {
                        eprintln!("Modal story: state changed; using current authored scene");
                        fresh["sceneSource"] = serde_json::json!("authored_fallback");
                        return Ok(Json(fresh));
                    }
                    if let Ok(scene) = proposal
                        && wonderwilds_director::story::validate_scene(
                            &snapshot["sceneInput"],
                            &snapshot["scene"],
                            &scene,
                        )
                        .is_ok()
                    {
                        snapshot["scene"] = scene;
                        snapshot["sceneSource"] = serde_json::json!("llm");
                        eprintln!("Modal story: validated scene accepted");
                    } else {
                        snapshot["sceneSource"] = serde_json::json!("authored_fallback");
                        eprintln!("Modal story: unavailable or invalid; using authored scene");
                    }
                }
                if snapshot.get("sceneSource").is_none() {
                    snapshot["sceneSource"] = serde_json::json!("authored");
                }
                wonderwilds_contracts::validate("StorySnapshot", &snapshot)
                    .map_err(|_| Error::Unavailable)?;
                Ok(Json(snapshot))
            }
        }
    })
    .await
    .map_err(|_| Error::Unavailable)?
}

pub(crate) async fn snapshot(
    App(state): App<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, Error> {
    run(state, headers, None).await
}

pub(crate) async fn command(
    App(state): App<AppState>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<Value>, Error> {
    let Json(body) = body.map_err(|_| Error::InvalidInput)?;
    run(state, headers, Some(body)).await
}
