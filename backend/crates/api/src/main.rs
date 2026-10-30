//! Local authored development API. Parent login is deliberately absent until a
//! reviewed provider is integrated; missing sessions fail closed.
use wonderwilds_api::game::http::{AppState, router};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = std::env::var("WONDERWILDS_DATABASE_URL").ok();
    let origin =
        std::env::var("WONDERWILDS_ORIGIN").unwrap_or_else(|_| "https://wonderwilds.local".into());
    let state = AppState::development(connection, origin)
        .map_err(|_| "Invalid authored API development configuration")?
        .with_modal_story()?;
    let port: u16 = std::env::var("WONDERWILDS_PORT")
        .unwrap_or_else(|_| "3001".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    eprintln!(
        "Wonderwilds authored development API listening on port {port}; parent identity provider remains unconfigured"
    );
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
