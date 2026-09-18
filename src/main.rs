use bf::{Error, Hub, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use uuid::Uuid;
#[derive(Parser)]
#[command(
    name = "bf",
    about = "Open or reconnect to your local BulletFarm workbench"
)]
struct Cli {
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    Demo {
        #[arg(long, default_value = "basic")]
        fixture: String,
    },
    Run {
        #[arg(trailing_var_arg = true)]
        goal: Vec<String>,
    },
    Take {
        mission: String,
        #[arg(long)]
        version: i64,
    },
    Stop {
        mission: String,
        #[arg(long)]
        version: i64,
    },
    Doctor,
    Serve {
        #[arg(long, default_value = "127.0.0.1:0")]
        bind: String,
        #[arg(long)]
        no_open: bool,
    },
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
        Some(Command::Demo { fixture }) => {
            let hub = Hub::open(&dir.join("demos").join(&fixture))?;
            let receipt = hub.run_fixture(&fixture)?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
            if receipt.check_result != "pass" || receipt.pr_number.is_none() {
                return Err(Error::CheckMissing("fixture failed".into()));
            }
        }
        Some(Command::Serve { bind, no_open }) => serve(dir, bind, !no_open).await?,
        command => {
            let endpoint = connect(&dir).await;
            if command.is_none() && endpoint.is_err() {
                return serve(dir, "127.0.0.1:0".into(), true).await;
            }
            let endpoint = match endpoint {
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
                        Error::StorageUnavailable(
                            "hub did not start; run bf serve for diagnostics".into(),
                        )
                    })?
                }
            };
            let token = login(&endpoint).await?;
            match command {
                None=>{let project=remember_current(&endpoint,&token).await.unwrap_or_default();open_browser(&format!("{}#token={}&project={}",endpoint.url,token,project));},
                Some(Command::Doctor)=>println!("{}",serde_json::to_string_pretty(&request(&endpoint,&token,"/v3/doctor",None).await?)?),
                Some(Command::Run{goal})=>send(&endpoint,&token,"run",None,None,json!({"goal":goal.join(" "),"project_id":current_project(&endpoint,&token).await?})).await?,
                Some(Command::Take{mission,version})=>send(&endpoint,&token,"take",Some(mission),Some(version),json!({})).await?,
                Some(Command::Stop{mission,version})=>send(&endpoint,&token,"stop",Some(mission),Some(version),json!({})).await?,
                _=>unreachable!(),
            }
        }
    }
    Ok(())
}
async fn current_project(e: &Endpoint, token: &str) -> Result<String> {
    remember_current(e, token).await
}
async fn remember_current(e: &Endpoint, token: &str) -> Result<String> {
    let cwd = std::env::current_dir()?;
    let root = bf::gitutil::git(&cwd, &["rev-parse", "--show-toplevel"])?;
    let base = bf::gitutil::git(&cwd, &["rev-parse", "HEAD"])?;
    let name = std::path::Path::new(&root)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let body = json!({"schema_version":3,"command_id":Uuid::new_v4().to_string(),"kind":"remember_project","target_id":null,"expected_version":null,"payload":{"path":root,"base_oid":base,"name":name}});
    let result = request(e, token, "/v3/commands", Some(body)).await?;
    result["result"]["project_id"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::InvalidContract("missing project identity".into()))
}
async fn send(
    e: &Endpoint,
    token: &str,
    kind: &str,
    target: Option<String>,
    version: Option<i64>,
    payload: Value,
) -> Result<()> {
    let body = json!({"schema_version":3,"command_id":Uuid::new_v4().to_string(),"kind":kind,"target_id":target,"expected_version":version,"payload":payload});
    println!(
        "{}",
        serde_json::to_string_pretty(&request(e, token, "/v3/commands", Some(body)).await?)?
    );
    Ok(())
}
fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| Error::Other(e.to_string()))
}
async fn connect(dir: &std::path::Path) -> Result<Endpoint> {
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
    let result = request(e, &e.bootstrap, "/v3/bootstrap", Some(json!({}))).await?;
    result["token"]
        .as_str()
        .map(str::to_owned)
        .ok_or(Error::AuthRequired)
}
async fn request(e: &Endpoint, token: &str, path: &str, body: Option<Value>) -> Result<Value> {
    let request = if let Some(body) = body {
        client()?.post(format!("{}{path}", e.url)).json(&body)
    } else {
        client()?.get(format!("{}{path}", e.url))
    };
    let response = request
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    let status = response.status();
    let result: Value = response
        .json()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    if !status.is_success() {
        return Err(Error::Other(
            result["message"]
                .as_str()
                .unwrap_or("hub request failed")
                .into(),
        ));
    }
    Ok(result)
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
    let project = std::env::current_dir()
        .ok()
        .and_then(|cwd| hub.register_project("owner-demo", &cwd).ok())
        .unwrap_or_default();
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
    let probes = hub.clone();
    tokio::task::spawn_blocking(move || {
        let _ = probes.probe_accounts();
    });
    let controller = hub.clone();
    tokio::spawn(async move {
        loop {
            let h = controller.clone();
            let result = tokio::task::spawn_blocking(move || h.drive()).await;
            if let Ok(Err(error)) = result {
                eprintln!("controller: {error}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    eprintln!("bf workbench: {url} (live execution blocked; fixture demo available)");
    if open {
        let token = hub.ensure_session("owner-demo")?;
        open_browser(&format!("{url}#token={token}&project={project}"));
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
