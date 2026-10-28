//! Local authored development API. Parent login is deliberately absent until a
//! reviewed provider is integrated; missing sessions fail closed.
use wonderwilds_api::game::http::{AppState, router};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = std::env::var("WONDERWILDS_DATABASE_URL").ok();
    let origin =
        std::env::var("WONDERWILDS_ORIGIN").unwrap_or_else(|_| "https://wonderwilds.local".into());
    let state = AppState::development(connection, origin)
        .map_err(|_| "Invalid authored API development configuration")?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    eprintln!(
        "Wonderwilds authored development API listening on port 3001; parent identity provider remains unconfigured"
    );
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
