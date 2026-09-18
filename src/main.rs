use bf::{Error, Hub, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "bf",
    about = "BulletFarm hub — doctor, serve, or open the workbench"
)]
struct Cli {
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Doctor,
    Serve {
        #[arg(long, default_value = "127.0.0.1:0")]
        bind: String,
        #[arg(long)]
        no_open: bool,
    },
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir = directory(&cli);
    match cli.command {
        None | Some(Command::Web) => serve(dir, "127.0.0.1:0".into(), true).await,
        Some(Command::Serve { bind, no_open }) => serve(dir, bind, !no_open).await,
        Some(Command::Doctor) => {
            let hub = Hub::open(&dir)?;
            println!("{}", serde_json::to_string_pretty(&hub.doctor())?);
            Ok(())
        }
    }
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
    eprintln!("bf workbench: {url}");
    if open {
        let token = hub.ensure_session("owner")?;
        let _ = std::process::Command::new("xdg-open")
            .arg(format!("{url}#token={token}"))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    axum::serve(listener, app)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    Ok(())
}
