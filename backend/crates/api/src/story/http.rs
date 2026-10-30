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
            Some(request) => state
                .story_repository
                .execute(&identity, &request)
                .map(Json),
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
                    let fresh = state.story_repository.read(&current_identity)?;
                    if fresh != snapshot {
                        eprintln!("Modal story: state changed; using current authored scene");
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
                        eprintln!("Modal story: validated scene accepted");
                    } else {
                        eprintln!("Modal story: unavailable or invalid; using authored scene");
                    }
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
