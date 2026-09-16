use bf::{Hub, Result};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "bf",
    about = "BulletFarm — ask, inspect, steer, take over",
    after_help = "Run `bf` with no arguments to start the local hub and workbench.\n`bf demo` is the fake, no-network proof. It is not a live-provider certification."
)]
struct Cli {
    /// Durable hub directory. Defaults to $HOME/.bf
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Deterministic fake loop. No keys, no network.
    Demo {
        #[arg(long, default_value = "basic")]
        fixture: String,
    },
    /// Describe a goal. Same path as the workbench composer.
    Run {
        #[arg(trailing_var_arg = true)]
        goal: Vec<String>,
    },
    /// Take over the current task as the human owner.
    Take,
    /// Stop dispatching new work.
    Stop,
    /// Model-free health. Speaks only when something is actually broken.
    Doctor,
    /// Start the hub (also the default when no subcommand is given).
    Serve {
        #[arg(long, default_value = "127.0.0.1:7420")]
        bind: String,
    },
}

fn data_dir(cli: &Cli) -> PathBuf {
    cli.data_dir.clone().unwrap_or_else(|| {
        dirs_home()
            .map(|h| h.join(".bf"))
            .unwrap_or_else(|| PathBuf::from(".bf"))
    })
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir = data_dir(&cli);
    match cli.command.unwrap_or(Command::Serve {
        bind: "127.0.0.1:7420".into(),
    }) {
        Command::Demo { fixture } => {
            let dir = tempfile_dir(&fixture)?;
            let hub = Hub::open(&dir)?;
            let receipt = hub.run_fixture(&fixture)?;
            println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
            if receipt.check_result != "pass" || receipt.pr_number.is_none() {
                std::process::exit(1);
            }
        }
        Command::Doctor => {
            let hub = Hub::open(&dir)?;
            println!("{}", serde_json::to_string_pretty(&hub.doctor()).unwrap());
        }
        Command::Run { goal } => {
            let hub = Hub::open(&dir)?;
            let text = goal.join(" ");
            let body = serde_json::json!({
                "schema_version": 3,
                "command_id": format!("run-{}", uuid::Uuid::new_v4()),
                "kind": "run",
                "target_id": null,
                "expected_version": null,
                "payload": {"goal": text}
            });
            let raw = serde_json::to_vec(&body).unwrap();
            let op = hub.command_bytes("owner-demo", &raw)?;
            println!("{}", serde_json::to_string_pretty(&op).unwrap());
        }
        Command::Take => {
            let hub = Hub::open(&dir)?;
            let body = serde_json::json!({
                "schema_version": 3,
                "command_id": format!("take-{}", uuid::Uuid::new_v4()),
                "kind": "take",
                "target_id": "T-001",
                "expected_version": 1,
                "payload": {"checkpoint_preference": "last_durable"}
            });
            let op = hub.command_bytes("owner-demo", &serde_json::to_vec(&body).unwrap())?;
            println!("{}", serde_json::to_string_pretty(&op).unwrap());
        }
        Command::Stop => {
            let hub = Hub::open(&dir)?;
            let body = serde_json::json!({
                "schema_version": 3,
                "command_id": format!("stop-{}", uuid::Uuid::new_v4()),
                "kind": "stop",
                "target_id": "M-001",
                "expected_version": 1,
                "payload": {"mission_id": "M-001"}
            });
            let op = hub.command_bytes("owner-demo", &serde_json::to_vec(&body).unwrap())?;
            println!("{}", serde_json::to_string_pretty(&op).unwrap());
        }
        Command::Serve { bind } => {
            serve(dir, bind).await?;
        }
    }
    Ok(())
}

fn tempfile_dir(fixture: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("bf-demo-{fixture}-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

async fn serve(dir: PathBuf, bind: String) -> Result<()> {
    let hub = Arc::new(Hub::open(&dir)?);
    let _ = hub.ensure_session("owner-demo");
    let web_dir = bf::crate_root().join("web/dist");
    let app = bf::api::router(bf::api::AppState { hub, web_dir });
    let addr: SocketAddr = bind
        .parse()
        .map_err(|e| bf::Error::Other(format!("bad bind {bind}: {e}")))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| bf::Error::Other(e.to_string()))?;
    let url = format!("http://{addr}");
    eprintln!("bf hub on {url}  (fake executor; doctor at /v3/doctor)");
    let _ = std::process::Command::new("xdg-open").arg(&url).status();
    axum::serve(listener, app)
        .await
        .map_err(|e| bf::Error::Other(e.to_string()))?;
    Ok(())
}
