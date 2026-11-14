use crate::game::{Error, http::AppState};
use axum::{Json, extract::State, http::HeaderMap};
use serde_json::Value;
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
        state
            .campaign_repository
            .run(&identity, request.as_ref())
            .map(Json)
    })
    .await
    .map_err(|_| Error::Unavailable)?
}
pub(crate) async fn snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, Error> {
    run(state, headers, None).await
}
pub(crate) async fn command(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<Value>, Error> {
    run(
        state,
        headers,
        Some(body.map_err(|_| Error::InvalidInput)?.0),
    )
    .await
}
