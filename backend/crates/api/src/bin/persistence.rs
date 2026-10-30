//! Offline operator maintenance; never exposed as a game/HTTP reset endpoint.
use wonderwilds_api::{game, story};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4
        || !["audit", "recover"].contains(&args[0].as_str())
        || !["game", "story"].contains(&args[1].as_str())
    {
        return Err("Usage: persistence <audit|recover> <game|story> <parent-uuid> <profile-uuid>; requires WONDERWILDS_DATABASE_URL".into());
    }
    let connection = std::env::var("WONDERWILDS_DATABASE_URL")
        .map_err(|_| "Explicit database connection required")?;
    let identity = game::Identity {
        parent_id: args[2].clone(),
        profile_id: args[3].clone(),
        session_id: "offline-maintenance".into(),
    };
    let result = if args[1] == "game" {
        let repo = game::repository::PostgresRepository::new(
            connection,
            game::development_content().map_err(|_| "Content unavailable")?,
        );
        if args[0] == "audit" {
            repo.verify_history(&identity)
        } else {
            repo.recover_current(&identity)
        }
    } else {
        let repo = story::repository::PostgresRepository::new(connection);
        if args[0] == "audit" {
            repo.verify_history(&identity)
        } else {
            repo.recover_current(&identity)
        }
    }
    .map_err(|error| format!("Persistence operation failed closed: {error:?}"))?;
    println!(
        "{} {}: verified revision {}",
        args[0], args[1], result["revision"]
    );
    Ok(())
}
