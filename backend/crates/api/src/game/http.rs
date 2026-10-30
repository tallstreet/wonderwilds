//! HTTP boundary. No login, test-session or provider bypass route is exposed.
use super::{
    repository::{PostgresRepository, Repository},
    *,
};
use crate::identity::{
    self,
    postgres::{PostgresIdentity, RepositoryError},
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State as App},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use sha2::{Digest, Sha256};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub trait Authenticator: Send + Sync {
    fn authenticate(&self, headers: &HeaderMap, write: bool, now: u64) -> Result<Identity, Error>;
}
struct DatabaseAuth {
    connection: String,
    origin: String,
}
impl Authenticator for DatabaseAuth {
    fn authenticate(&self, headers: &HeaderMap, write: bool, now: u64) -> Result<Identity, Error> {
        let cookies = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .ok_or(Error::Unauthenticated)?;
        let tokens: Vec<_> = cookies
            .split(';')
            .filter_map(|part| part.trim().strip_prefix("wonderwilds_session="))
            .collect();
        if tokens.len() != 1 || tokens[0].len() < 32 || tokens[0].len() > 256 {
            return Err(Error::Unauthenticated);
        }
        let token: [u8; 32] = Sha256::digest(tokens[0].as_bytes()).into();
        let profile = headers
            .get("x-wonderwilds-profile")
            .and_then(|v| v.to_str().ok())
            .filter(|v| v.len() == 36)
            .ok_or(Error::Forbidden)?;
        let csrf = if write {
            let raw = headers
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .filter(|v| v.len() >= 32 && v.len() <= 256)
                .ok_or(Error::Forbidden)?;
            Some(<[u8; 32]>::from(Sha256::digest(raw.as_bytes())))
        } else {
            None
        };
        let origin = headers.get("origin").and_then(|v| v.to_str().ok());
        let client = postgres::Client::connect(&self.connection, postgres::NoTls)
            .map_err(|_| Error::Unavailable)?;
        let verified = PostgresIdentity::new(client)
            .authenticate_context(&token, csrf.as_ref(), origin, &self.origin, profile, now)
            .map_err(|error| match error {
                RepositoryError::Storage(_) => Error::Unavailable,
                RepositoryError::Access(error) => match error {
                    identity::Error::Unauthenticated => Error::Unauthenticated,
                    identity::Error::Forbidden | identity::Error::Deleted => Error::Forbidden,
                    identity::Error::InvalidInput => Error::InvalidInput,
                },
            })?;
        Ok(Identity {
            parent_id: verified.parent_id,
            profile_id: verified.profile_id,
            session_id: format!("session-{}", digest_id(tokens[0])),
        })
    }
}
struct DisabledAuth;
impl Authenticator for DisabledAuth {
    fn authenticate(&self, _: &HeaderMap, _: bool, _: u64) -> Result<Identity, Error> {
        Err(Error::Unauthenticated)
    }
}
struct DisabledRepository;
impl Repository for DisabledRepository {
    fn read(&self, _: &Identity) -> Result<Aggregate, Error> {
        Err(Error::Unavailable)
    }
    fn execute(&self, _: &Identity, _: &Value, _: u64) -> Result<Value, Error> {
        Err(Error::Unavailable)
    }
}
struct DisabledStoryRepository;
impl crate::story::repository::Repository for DisabledStoryRepository {
    fn read(&self, _: &Identity) -> Result<Value, Error> {
        Err(Error::Unavailable)
    }
    fn execute(&self, _: &Identity, _: &Value) -> Result<Value, Error> {
        Err(Error::Unavailable)
    }
}
struct DisabledMissionLearningRepository;
impl crate::mission_learning::repository::Repository for DisabledMissionLearningRepository {
    fn readiness(&self) -> Result<(), Error> {
        Err(Error::Unavailable)
    }
    fn read(&self, _: &Identity) -> Result<Value, Error> {
        Err(Error::Unavailable)
    }
    fn execute(&self, _: &Identity, _: &Value) -> Result<Value, Error> {
        Err(Error::Unavailable)
    }
}
#[derive(Clone)]
pub struct AppState {
    repository: Arc<dyn Repository>,
    pub(crate) auth: Arc<dyn Authenticator>,
    pub(crate) story_provider:
        Option<Arc<dyn wonderwilds_director::story::SceneProvider + Send + Sync>>,
    pub(crate) story_explorer: Option<Arc<dyn crate::story::generation::ExplorationProvider>>,
    pub(crate) story_planner: Option<Arc<dyn crate::story::planning::PlanProvider>>,
    pub(crate) story_repository: Arc<dyn crate::story::repository::Repository>,
    pub(crate) mission_learning_repository:
        Arc<dyn crate::mission_learning::repository::Repository>,
    content: Content,
    pub(crate) permits: Arc<tokio::sync::Semaphore>,
    configured: bool,
}
impl AppState {
    /// Explicit opt-in; credentials remain in the backend process only.
    pub fn with_modal_story(mut self) -> Result<Self, &'static str> {
        self.story_provider =
            crate::story::modal::ModalSceneProvider::from_env()?.map(|provider| {
                Arc::new(provider)
                    as Arc<dyn wonderwilds_director::story::SceneProvider + Send + Sync>
            });
        self.story_explorer = crate::story::modal::ModalSceneProvider::explorer_from_env()?
            .map(|p| Arc::new(p) as Arc<dyn crate::story::generation::ExplorationProvider>);
        self.story_planner = crate::story::modal::ModalSceneProvider::planner_from_env()?
            .map(|p| Arc::new(p) as Arc<dyn crate::story::planning::PlanProvider>);
        if self.story_explorer.is_some() {
            eprintln!("Generated neighbourhood stories enabled for development arrivals");
        }
        if self.story_planner.is_some() {
            eprintln!("Modal mission planning enabled for new story acceptances");
        }
        if self.story_provider.is_some() {
            eprintln!("Modal story generation enabled for development scene reads");
        }
        Ok(self)
    }

    /// Development server supports private Unix-socket PostgreSQL only. A
    /// production adapter must configure verified DB TLS and an identity issuer.
    pub fn development(connection: Option<String>, origin: String) -> Result<Self, Error> {
        let content = development_content()?;
        let story_repository: Arc<dyn crate::story::repository::Repository> = match &connection {
            Some(value) => Arc::new(crate::story::repository::PostgresRepository::new(
                value.clone(),
            )),
            None => Arc::new(DisabledStoryRepository),
        };
        let mission_learning_repository: Arc<dyn crate::mission_learning::repository::Repository> =
            match &connection {
                Some(value) => Arc::new(
                    crate::mission_learning::repository::PostgresRepository::new(value.clone()),
                ),
                None => Arc::new(DisabledMissionLearningRepository),
            };
        let (repository, auth, configured): (Arc<dyn Repository>, Arc<dyn Authenticator>, bool) =
            if let Some(connection) = connection {
                let config: postgres::Config =
                    connection.parse().map_err(|_| Error::InvalidInput)?;
                if config.get_hosts().is_empty()
                    || !config.get_hostaddrs().is_empty()
                    || config
                        .get_hosts()
                        .iter()
                        .any(|host| !matches!(host, postgres::config::Host::Unix(_)))
                {
                    return Err(Error::InvalidInput);
                }
                (
                    Arc::new(PostgresRepository::new(connection.clone(), content.clone())),
                    Arc::new(DatabaseAuth { connection, origin }),
                    true,
                )
            } else {
                (Arc::new(DisabledRepository), Arc::new(DisabledAuth), false)
            };
        Ok(Self {
            repository,
            auth,
            story_repository,
            mission_learning_repository,
            story_provider: None,
            story_planner: None,
            story_explorer: None,
            content,
            permits: Arc::new(tokio::sync::Semaphore::new(16)),
            configured,
        })
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let (status, code) = match self {
            Self::InvalidInput => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),
            Self::Unauthenticated => (StatusCode::UNAUTHORIZED, "UNAUTHENTICATED"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::RevisionConflict => (StatusCode::CONFLICT, "REVISION_CONFLICT"),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            Self::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE"),
        };
        (status,Json(json!({"schemaVersion":"1.0","code":code,"correlationId":format!("api-{}",NEXT.fetch_add(1,Ordering::Relaxed))}))).into_response()
    }
}
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/healthz",
            get(|| async { Json(json!({"status":"ok","mode":"authored-development"})) }),
        )
        .route("/readyz", get(ready))
        .route("/api/v1/story", get(crate::story::http::snapshot))
        .route("/api/v1/story/commands", post(crate::story::http::command))
        .route(
            "/api/v1/mission-learning",
            get(crate::mission_learning::http::snapshot),
        )
        .route(
            "/api/v1/mission-learning/commands",
            post(crate::mission_learning::http::command),
        )
        .route("/api/v1/game", get(snapshot))
        .route("/api/v1/commands", post(command))
        .route("/api/v1/encounters/{id}", get(encounter))
        .route("/api/v1/encounters/{id}/attempts", post(attempt))
        .route("/api/v1/encounters/{id}/hints", post(hint))
        .fallback(|| async { Error::NotFound })
        .layer(DefaultBodyLimit::max(32768))
        .with_state(state)
}
async fn ready(App(state): App<AppState>) -> Result<Json<Value>, Error> {
    if !state.configured {
        return Err(Error::Unavailable);
    }
    let permit = state
        .permits
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::RateLimited)?;
    tokio::task::spawn_blocking(move||{let _permit=permit;state.repository.readiness()?;state.story_repository.readiness()?;state.mission_learning_repository.readiness()?;Ok(Json(json!({"status":"ready","mode":"authored-development","releaseReady":false,"contentReview":"pending"}))) }).await.map_err(|_|Error::Unavailable)?
}
fn now() -> Result<u64, Error> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|n| n.as_secs())
        .map_err(|_| Error::Unavailable)
}
async fn run(
    state: AppState,
    headers: HeaderMap,
    write: bool,
    request: Option<Value>,
    encounter_id: Option<String>,
) -> Result<Json<Value>, Error> {
    let permit = state
        .permits
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::RateLimited)?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let timestamp = now()?;
        let identity = state.auth.authenticate(&headers, write, timestamp)?;
        if let Some(request) = request {
            state
                .repository
                .execute(&identity, &request, timestamp)
                .map(Json)
        } else {
            let aggregate = state.repository.read(&identity)?;
            match encounter_id {
                Some(id) => aggregate.public_encounter(&id).map(Json),
                None => aggregate.snapshot(&state.content).map(Json),
            }
        }
    })
    .await
    .map_err(|_| Error::Unavailable)?
}
type Body = Result<Json<Value>, axum::extract::rejection::JsonRejection>;
async fn snapshot(App(state): App<AppState>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    run(state, headers, false, None, None).await
}
async fn encounter(
    App(state): App<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, Error> {
    run(state, headers, false, None, Some(id)).await
}
async fn command(
    App(state): App<AppState>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<Value>, Error> {
    let Json(body) = body.map_err(|_| Error::InvalidInput)?;
    run(state, headers, true, Some(body), None).await
}
async fn route_command(
    state: AppState,
    headers: HeaderMap,
    id: String,
    body: Body,
    kind: &str,
) -> Result<Json<Value>, Error> {
    let Json(body) = body.map_err(|_| Error::InvalidInput)?;
    if body["payload"]["type"] != kind || body["payload"]["encounterId"] != id {
        return Err(Error::InvalidInput);
    }
    run(state, headers, true, Some(body), None).await
}
async fn attempt(
    App(state): App<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Body,
) -> Result<Json<Value>, Error> {
    route_command(state, headers, id, body, "attempt").await
}
async fn hint(
    App(state): App<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Body,
) -> Result<Json<Value>, Error> {
    route_command(state, headers, id, body, "hint").await
}

#[cfg(test)]
mod tests;
