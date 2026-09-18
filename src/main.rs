//! `bf` — see and steer every coding agent on this host. Bare `bf` opens the screen; the other
//! verbs are one-liners agents run from any shell. Exit codes: 0 ok · 1 error · 2 conflict or
//! refusal (overlapping claim, stale claim, denied stop) · 3 not implemented yet · 64 bad input.
use bf::{Error, Hub, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "bf",
    version,
    about = "See and steer every coding agent on this host. `bf` alone opens the screen.",
    after_help = "Agents coordinate with: bf board · bf claim <paths> -m \"why\" · bf heartbeat <id> · bf release <id> --proof '<cmd>' · bf note -m \"…\" --to <agent>"
)]
struct Cli {
    /// Data directory (default ~/.bf)
    #[arg(long, global = true, env = "BF_DATA_DIR")]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Plain table of every live Claude/Codex/Cursor/Grok session and bf job
    Agents,
    /// Print the board digest (active claims, live agents, recent entries) and regenerate AGENT_CHAT.md
    Board {
        /// Print the full history instead of the last 30 entries
        #[arg(long)]
        all: bool,
        /// Only notes addressed to this agent (`me` = the caller's detected identity)
        #[arg(long)]
        to: Option<String>,
    },
    /// Claim repo-relative paths before editing them (exit 2 on overlap, naming the holder)
    Claim {
        /// Files or directories (a directory ends with `/`)
        #[arg(required = true)]
        paths: Vec<String>,
        /// What and why
        #[arg(short = 'm', long = "message")]
        message: String,
        /// Repository root (default: `git rev-parse --show-toplevel`, else the cwd)
        #[arg(long)]
        repo: Option<PathBuf>,
        /// Lease length, e.g. 30m or 2h (max 3h); heartbeat to extend
        #[arg(long, default_value = "30m")]
        ttl: String,
        /// Declared agent name (default: detected from the process tree)
        #[arg(long = "as", env = "BF_AGENT")]
        agent: Option<String>,
    },
    /// Extend a claim you hold
    Heartbeat {
        claim_id: String,
        #[arg(short = 'm', long = "message", default_value = "")]
        message: String,
        #[arg(long = "as", env = "BF_AGENT")]
        agent: Option<String>,
    },
    /// Release a claim with proof: `-m "<what you ran>"` or `--proof '<command>'` (runs it, records the exit code)
    Release {
        claim_id: String,
        #[arg(short = 'm', long = "message")]
        message: Option<String>,
        #[arg(long)]
        proof: Option<String>,
        #[arg(long = "as", env = "BF_AGENT")]
        agent: Option<String>,
    },
    /// Post a note addressed to an agent (--to), a claim (--re) or a PR (--pr); unaddressed = short status line
    Note {
        #[arg(short = 'm', long = "message")]
        message: String,
        #[arg(long)]
        to: Option<String>,
        #[arg(long = "re")]
        re: Option<String>,
        #[arg(long)]
        pr: Option<String>,
        #[arg(long = "as", env = "BF_AGENT")]
        agent: Option<String>,
    },
    /// Stop a session's process group (SIGTERM, then SIGKILL) and record it
    Stop { pid: i64 },
    /// Open pull requests across the repositories seen on the board
    Prs,
    /// Dispatch a headless agent in the current directory
    Run {
        /// claude | codex | cursor | grok
        provider: String,
        #[arg(trailing_var_arg = true, required = true)]
        prompt: Vec<String>,
        /// Allow the provider to run without permission prompts
        #[arg(long)]
        yolo: bool,
    },
    /// Read-only health report: data dir, SQLite, gh auth, provider and toolchain versions
    Doctor,
    /// Run the loopback HTTP hub for the browser page (single instance)
    Serve {
        #[arg(long, default_value = "127.0.0.1:0")]
        bind: String,
        #[arg(long)]
        no_open: bool,
    },
    /// Start the hub if needed and open (or print) the browser page URL
    Web,
}

#[derive(Serialize, Deserialize)]
struct Endpoint {
    url: String,
    bootstrap: String,
    epoch: String,
}

fn directory(cli: &Cli) -> PathBuf {
    cli.data_dir.clone().unwrap_or_else(|| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".bf")
    })
}

fn exit_code(error: &Error) -> u8 {
    match error {
        Error::ResourceConflict(_) | Error::StaleVersion | Error::PolicyDenied(_) => 2,
        Error::Other(m) if m.starts_with("not implemented") => 3,
        Error::InvalidContract(_) => 64,
        _ => 1,
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bf: {error}");
            ExitCode::from(exit_code(&error))
        }
    }
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Other(e.to_string()))
}

fn dispatch(cli: Cli) -> Result<()> {
    let dir = directory(&cli);
    match cli.command {
        None => bf::tui::run(Box::new(bf::tui::StubSource)),
        Some(Command::Agents) => {
            let agents = bf::agents::discover(&bf::agents::Env::from_env())?;
            print!("{}", bf::agents::plain_table(&agents));
            Ok(())
        }
        Some(Command::Board { all, to }) => {
            print!("{}", bf::board::cli_board(all, to.as_deref())?);
            Ok(())
        }
        Some(Command::Claim {
            paths,
            message,
            repo,
            ttl,
            agent,
        }) => {
            let ttl = bf::board::parse_ttl(&ttl)?;
            println!(
                "{}",
                bf::board::cli_claim(
                    &paths,
                    &message,
                    repo.as_deref().and_then(Path::to_str),
                    ttl,
                    agent.as_deref()
                )?
            );
            Ok(())
        }
        Some(Command::Heartbeat {
            claim_id,
            message,
            agent,
        }) => {
            println!(
                "{}",
                bf::board::cli_heartbeat(&claim_id, &message, agent.as_deref())?
            );
            Ok(())
        }
        Some(Command::Release {
            claim_id,
            message,
            proof,
            agent,
        }) => {
            if message.is_none() && proof.is_none() {
                return Err(Error::InvalidContract(
                    "release needs -m \"<proof>\" or --proof '<command>'".into(),
                ));
            }
            println!(
                "{}",
                bf::board::cli_release(
                    &claim_id,
                    message.as_deref(),
                    proof.as_deref(),
                    agent.as_deref()
                )?
            );
            Ok(())
        }
        Some(Command::Note {
            message,
            to,
            re,
            pr,
            agent,
        }) => {
            println!(
                "{}",
                bf::board::cli_note(
                    &message,
                    to.as_deref(),
                    re.as_deref(),
                    pr.as_deref(),
                    agent.as_deref()
                )?
            );
            Ok(())
        }
        Some(Command::Stop { pid: _ }) => {
            Err(Error::Other("not implemented yet (plan PR 7: stop)".into()))
        }
        Some(Command::Prs) => {
            let prs = bf::prs::list(&[])?;
            println!("{}", serde_json::to_string_pretty(&prs)?);
            Ok(())
        }
        Some(Command::Run {
            provider,
            prompt: _,
            yolo: _,
        }) => {
            if !["claude", "codex", "cursor", "grok"].contains(&provider.as_str()) {
                return Err(Error::InvalidContract(format!(
                    "unknown provider {provider}; use claude|codex|cursor|grok"
                )));
            }
            Err(Error::Other(
                "not implemented yet (plan PR 9: bf run)".into(),
            ))
        }
        Some(Command::Doctor) => {
            println!("{}", serde_json::to_string_pretty(&doctor(&dir))?);
            Ok(())
        }
        Some(Command::Serve { bind, no_open }) => runtime()?.block_on(serve(dir, bind, !no_open)),
        Some(Command::Web) => runtime()?.block_on(web(dir)),
    }
}

/// Run `cmd args…` and return its first non-empty output line, or why it failed. Never blocks on input.
fn probe(cmd: &str, args: &[&str]) -> Value {
    match std::process::Command::new(cmd)
        .args(args)
        .env("TERM", "dumb")
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(out) => {
            let text = if out.stdout.is_empty() {
                &out.stderr
            } else {
                &out.stdout
            };
            let line = String::from_utf8_lossy(text)
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("")
                .to_string();
            json!({"ok": out.status.success(), "text": line})
        }
        Err(e) => json!({"ok": false, "text": format!("{cmd}: {e}")}),
    }
}

/// Read-only: takes no lock and opens no database, so it works while `bf serve` runs.
fn doctor(dir: &Path) -> Value {
    let node_pin = include_str!("../web/.nvmrc").trim();
    json!({
        "data_dir": dir,
        "data_dir_exists": dir.exists(),
        "hub_endpoint": dir.join("endpoint.json").exists(),
        "sqlite": rusqlite::version(),
        "toolchain": {
            "node": probe("node", &["--version"]), "node_pin": format!("v{node_pin}"),
            "npm": probe("npm", &["--version"]), "npm_pin": "10.9.8",
            "cargo": probe("cargo", &["--version"]),
        },
        "providers": {
            "claude": probe("claude", &["--version"]),
            "codex": probe("codex", &["--version"]),
            "cursor-agent": probe("cursor-agent", &["--version"]),
            "grok": probe("grok", &["--version"]),
        },
        "gh": probe("gh", &["auth", "status", "-h", "github.com"]),
        "identity": bf::identity::current(None),
    })
}

async fn web(dir: PathBuf) -> Result<()> {
    let endpoint = match connect(&dir).await {
        Ok(e) => e,
        Err(_) => {
            std::process::Command::new(std::env::current_exe()?)
                .arg("--data-dir")
                .arg(&dir)
                .args(["serve", "--no-open"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
            let mut ready = None;
            for _ in 0..50 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Ok(e) = connect(&dir).await {
                    ready = Some(e);
                    break;
                }
            }
            ready.ok_or_else(|| {
                Error::StorageUnavailable("hub did not start; run bf serve for diagnostics".into())
            })?
        }
    };
    let token = login(&endpoint).await?;
    let url = format!("{}#token={}", endpoint.url, token);
    if std::env::var_os("DISPLAY").is_some() {
        open_browser(&url);
    }
    println!("{url}");
    Ok(())
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| Error::Other(e.to_string()))
}

async fn connect(dir: &Path) -> Result<Endpoint> {
    let e: Endpoint = serde_json::from_slice(&std::fs::read(dir.join("endpoint.json"))?)?;
    let url =
        reqwest::Url::parse(&e.url).map_err(|_| Error::InvalidContract("bad endpoint".into()))?;
    if url.scheme() != "http" || url.host_str() != Some("127.0.0.1") || url.port().is_none() {
        return Err(Error::PolicyDenied(
            "endpoint must be explicit IPv4 loopback".into(),
        ));
    }
    let response = client()?
        .get(format!("{}/health", e.url))
        .send()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    let body: Value = response
        .json()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    if body["service"] != "bf" {
        return Err(Error::AuthRequired);
    }
    Ok(e)
}

async fn login(e: &Endpoint) -> Result<String> {
    let response = client()?
        .post(format!("{}/v3/bootstrap", e.url))
        .bearer_auth(&e.bootstrap)
        .json(&json!({}))
        .send()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    let result: Value = response
        .json()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    result["token"]
        .as_str()
        .map(str::to_owned)
        .ok_or(Error::AuthRequired)
}

async fn serve(dir: PathBuf, bind: String, open: bool) -> Result<()> {
    let addr: SocketAddr = bind
        .parse()
        .map_err(|_| Error::InvalidContract("invalid bind".into()))?;
    if !addr.ip().is_loopback() || !addr.is_ipv4() {
        return Err(Error::PolicyDenied(
            "this private pilot only binds IPv4 loopback".into(),
        ));
    }
    let hub = Arc::new(Hub::open(&dir)?);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let url = format!("http://{}", listener.local_addr()?);
    let bootstrap = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let endpoint = Endpoint {
        url: url.clone(),
        bootstrap: bootstrap.clone(),
        epoch: hub.epoch.clone(),
    };
    let path = dir.join("endpoint.json");
    {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&path)?;
        file.write_all(&serde_json::to_vec(&endpoint)?)?;
        file.sync_all()?;
    }
    let app = bf::api::router(bf::api::AppState::new(hub.clone(), url.clone(), bootstrap));
    eprintln!("bf hub: {url}");
    if open {
        let token = hub.ensure_session("owner")?;
        open_browser(&format!("{url}#token={token}"));
    }
    axum::serve(listener, app)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    Ok(())
}

fn open_browser(url: &str) {
    let _ = std::process::Command::new("xdg-open")
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
