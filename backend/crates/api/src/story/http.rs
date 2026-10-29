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
            None => state.story_repository.read(&identity).map(Json),
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
