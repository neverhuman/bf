use crate::hub::Hub;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub hub: Arc<Hub>,
    pub web_dir: PathBuf,
}

pub fn router(state: AppState) -> Router {
    let static_files = ServeDir::new(&state.web_dir);
    Router::new()
        .route("/health", get(health))
        .route("/v3/doctor", get(doctor))
        .route("/v3/work", get(work))
        .route("/v3/commands", post(commands))
        .route("/v3/operations/{id}", get(operation))
        .route("/v3/demo/{fixture}", post(demo))
        .route("/", get(index))
        .fallback_service(static_files)
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({"ok": true}))
}

async fn doctor(State(state): State<AppState>) -> Json<Value> {
    Json(state.hub.doctor())
}

async fn work(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, crate::Error> {
    require_actor(&state, &headers)?;
    let items = state.hub.work()?;
    Ok(Json(json!({"items": items})))
}

async fn commands(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), crate::Error> {
    let actor = require_actor(&state, &headers)?;
    let raw = serde_json::to_vec(&body)?;
    let op = state.hub.command_bytes(&actor, &raw)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"operation_id": op.id, "status": op.status, "result": op.result})),
    ))
}

async fn operation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, crate::Error> {
    require_actor(&state, &headers)?;
    let items = state.hub.work()?;
    Ok(Json(json!({"id": id, "work": items})))
}

async fn demo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(fixture): Path<String>,
) -> Result<Json<Value>, crate::Error> {
    require_actor(&state, &headers)?;
    let receipt = state.hub.run_fixture(&fixture)?;
    Ok(Json(serde_json::to_value(receipt)?))
}

async fn index(State(state): State<AppState>) -> Response {
    let built = state.web_dir.join("index.html");
    if let Ok(html) = std::fs::read_to_string(built) {
        return Html(html).into_response();
    }
    Html(FALLBACK_HTML).into_response()
}

fn require_actor(state: &AppState, headers: &HeaderMap) -> Result<String, crate::Error> {
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw);
    if token.is_empty() {
        return Ok("owner-demo".into());
    }
    state.hub.lookup_session(token).or_else(|_| {
        if token == state.hub.token {
            Ok("owner-demo".into())
        } else {
            Err(crate::Error::AuthRequired)
        }
    })
}

const FALLBACK_HTML: &str = r#"<!doctype html>
<html lang="en">
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>BulletFarm</title>
<style>
  :root { color-scheme: dark; }
  body { margin: 0; font: 16px/1.45 ui-sans-serif, system-ui; background: #0c0d10; color: #f4f1e8; }
  main { max-width: 720px; margin: 12vh auto; padding: 0 24px; }
  h1 { font-weight: 650; letter-spacing: -0.03em; }
  textarea { width: 100%; min-height: 90px; background: #17181d; color: inherit; border: 1px solid #2c2e36; border-radius: 12px; padding: 14px; font: inherit; }
  button { margin-top: 12px; background: #d6ff3f; color: #111; border: 0; border-radius: 999px; padding: 10px 18px; font-weight: 700; cursor: pointer; }
  pre { background: #17181d; padding: 16px; border-radius: 12px; overflow: auto; }
  .muted { color: #9aa0ad; }
</style>
<main>
  <p class="muted">ask · inspect · steer · take over</p>
  <h1>What should happen?</h1>
  <textarea id="goal" placeholder="Reject repeated delivery IDs"></textarea>
  <div>
    <button id="run">Run demo</button>
    <button id="work" style="background:#2c2e36;color:#f4f1e8">Show work</button>
  </div>
  <pre id="out" class="muted">Status needs no model.</pre>
</main>
<script>
const out = document.getElementById('out');
document.getElementById('run').onclick = async () => {
  const r = await fetch('/v3/demo/basic', {method:'POST', headers:{'Content-Type':'application/json'}});
  out.textContent = await r.text();
};
document.getElementById('work').onclick = async () => {
  const r = await fetch('/v3/work');
  out.textContent = await r.text();
};
</script>
</html>"#;
