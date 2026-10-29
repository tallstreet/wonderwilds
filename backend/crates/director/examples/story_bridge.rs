//! Local, synthetic headless journey. No HTTP, database, provider or real profile.
use serde_json::{Value, json};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use wonderwilds_director::story::{
    authored_plan, authored_scene, build_plan_input, build_scene_input, resolve_action,
};
use wonderwilds_domain::story::{Action, Command, MissionStatus, State};

const CHECKPOINT_BYTES: u64 = 256 * 1024;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Default)]
struct Options {
    output: Option<PathBuf>,
    resume: Option<PathBuf>,
    choose: Option<String>,
    workshop_source: bool,
}

fn options() -> Result<Options> {
    let mut result = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output-dir" => {
                result.output = Some(args.next().ok_or("--output-dir needs a path")?.into());
            }
            "--resume" => {
                result.resume = Some(
                    args.next()
                        .ok_or("--resume needs a checkpoint path")?
                        .into(),
                );
            }
            "--choose" => {
                result.choose = Some(args.next().ok_or("--choose needs an action handle")?);
            }
            "--source" => match args.next().as_deref() {
                Some("workshop") => result.workshop_source = true,
                Some("bank") => result.workshop_source = false,
                _ => return Err("--source must be bank or workshop".into()),
            },
            _ => {
                return Err(format!(
                    "Unknown argument {arg}. Usage: story_bridge [--source bank|workshop] [--output-dir NEW_DIR] [--resume CHECKPOINT [--choose HANDLE]]"
                )
                .into());
            }
        }
    }
    if result.choose.is_some() && result.resume.is_none() {
        return Err("--choose requires --resume; use an action handle from its scene input".into());
    }
    Ok(result)
}

fn load(path: &Path) -> Result<State> {
    // Bound bytes before JSON allocation. Replay checks consistency, not authenticity.
    let mut bytes = Vec::new();
    File::open(path)?
        .take(CHECKPOINT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > CHECKPOINT_BYTES {
        return Err("Checkpoint exceeds 256 KiB".into());
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    State::from_checkpoint(&value).map_err(|e| format!("Invalid checkpoint: {e:?}").into())
}

fn inspect(state: &State) -> Result<Value> {
    let plan_input = build_plan_input(state).map_err(|e| format!("Plan input: {e:?}"))?;
    let scene_input = build_scene_input(state).map_err(|e| format!("Scene input: {e:?}"))?;
    let plan = authored_plan(state).map_err(|e| format!("Authored plan: {e:?}"))?;
    let scene = authored_scene(state).map_err(|e| format!("Authored scene: {e:?}"))?;
    Ok(json!({
        "contentVersion": wonderwilds_domain::story::CONTENT_VERSION,
        "stateRevision": state.revision(),
        "planningInput": plan_input,
        "authoredPlan": plan,
        "sceneInput": scene_input,
        "authoredScene": scene
    }))
}

fn record(state: &State, output: Option<&Path>) -> Result<Value> {
    let view = inspect(state)?;
    let checkpoint = state.checkpoint();
    let resumed = State::from_checkpoint(&checkpoint)
        .map_err(|e| format!("Generated checkpoint failed replay: {e:?}"))?;
    if inspect(&resumed)? != view {
        return Err("Payload changed after checkpoint replay".into());
    }
    if let Some(dir) = output {
        fs::write(
            dir.join(format!("{:03}-payload.json", state.revision())),
            serde_json::to_vec_pretty(&view)?,
        )?;
        fs::write(
            dir.join(format!("{:03}-checkpoint.json", state.revision())),
            serde_json::to_vec_pretty(&checkpoint)?,
        )?;
    }
    Ok(view)
}

fn execute(state: &mut State, action: Action) -> Result<()> {
    let command = Command {
        command_id: format!("demo-{}", state.revision() + 1),
        expected_revision: state.revision(),
        action,
    };
    let receipt = state
        .apply(&command)
        .map_err(|e| format!("Action rejected: {e:?}"))?;
    let checkpoint = state.checkpoint();
    // Exercise uncertain-write retry: same body/ID must return the original receipt.
    if state.apply(&command) != Ok(receipt.clone()) || state.checkpoint() != checkpoint {
        return Err("Exact command retry changed state or response".into());
    }
    eprintln!("{}: {}", receipt.revision, receipt.description);
    Ok(())
}

fn travel(to: &str) -> Action {
    Action::Travel {
        destination_id: to.into(),
    }
}

fn main() -> Result<()> {
    let options = options()?;
    let mut state = match &options.resume {
        Some(path) => load(path)?,
        None => State::new(),
    };
    // Require a new directory to avoid overwriting previous traces or user data.
    if let Some(dir) = &options.output {
        fs::create_dir(dir)?;
    }
    let output = options.output.as_deref();
    if options.resume.is_some() {
        if let Some(handle) = options.choose {
            let action = resolve_action(&state, &handle)
                .map_err(|e| format!("Invalid or stale action handle: {e:?}"))?;
            execute(&mut state, action)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&record(&state, output)?)?
        );
        return Ok(());
    }

    record(&state, output)?;
    execute(&mut state, Action::AcceptMission)?;
    record(&state, output)?;
    if options.workshop_source {
        execute(&mut state, travel("workshop"))?;
        record(&state, output)?;
    }
    execute(
        &mut state,
        Action::Collect {
            source_id: if options.workshop_source {
                "spare_wood"
            } else {
                "fallen_wood"
            }
            .into(),
            item_id: "wood".into(),
            quantity: 3,
        },
    )?;
    record(&state, output)?;
    if !options.workshop_source {
        execute(&mut state, travel("workshop"))?;
        record(&state, output)?;
    }

    let presented = state.presentation();
    execute(
        &mut state,
        Action::Acknowledge {
            scene_id: presented.scene_id,
            entry_id: presented.entries[0].entry_id.clone(),
        },
    )?;
    record(&state, output)?;
    // Reload at the workshop. Rebuild handles after acknowledgement changed revision.
    state = State::from_checkpoint(&state.checkpoint())
        .map_err(|e| format!("Workshop resume failed: {e:?}"))?;
    for action in [
        Action::Exchange {
            character_id: "woodcutter".into(),
            recipe_id: "wood_to_planks".into(),
        },
        travel("bridge_bank"),
        Action::Assemble {
            object_id: "bridge".into(),
            recipe_id: "repair_bridge".into(),
        },
        Action::Accompany {
            character_id: "pip".into(),
            destination_id: "far_bank".into(),
        },
    ] {
        execute(&mut state, action)?;
        record(&state, output)?;
    }
    if state.mission_status() != MissionStatus::Completed
        || !state.bridge_repaired()
        || !state.crossed_with_pip()
        || state.wood() != 0
        || state.planks() != 0
    {
        return Err("Journey did not achieve the declared outcome with conserved inventory".into());
    }
    eprintln!(
        "Authored mission complete; model disabled. Each checkpoint replay preserved its payload."
    );
    println!("{}", serde_json::to_string_pretty(&inspect(&state)?)?);
    Ok(())
}
