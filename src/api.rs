use crate::{Error, Hub, Result};
use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Semaphore;
include!(concat!(env!("OUT_DIR"), "/web_assets.rs"));

#[derive(Clone)]
pub struct AppState {
    pub hub: Arc<Hub>,
    origin: String,
    bootstrap: String,
    slots: Arc<Semaphore>,
}

impl AppState {
    pub fn new(hub: Arc<Hub>, origin: String, bootstrap: String) -> Self {
        Self {
            hub,
            origin,
            bootstrap,
            slots: Arc::new(Semaphore::new(32)),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(json!({"service":"bf","ok":true})) }),
        )
        .route("/v3/bootstrap", post(bootstrap))
        .route("/v3/session", get(session).delete(logout))
        .route("/v3/doctor", get(doctor))
        .route("/v3/commands", post(commands))
        .route("/v3/operations/{id}", get(operation))
        .fallback(get(asset))
        .with_state(state)
}

fn credential(state: &AppState, headers: &HeaderMap) -> Result<String> {
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if state.origin != format!("http://{host}") {
        return Err(Error::PolicyDenied("unexpected Host".into()));
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        if origin.to_str().ok() != Some(&state.origin) {
            return Err(Error::PolicyDenied("cross-origin request denied".into()));
        }
    }
    if headers
        .get("sec-fetch-site")
        .is_some_and(|h| h == "cross-site")
    {
        return Err(Error::PolicyDenied("cross-site request denied".into()));
    }
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .ok_or(Error::AuthRequired)
}

async fn call<T: Send + 'static>(
    state: AppState,
    headers: HeaderMap,
    f: impl FnOnce(&Hub, &str) -> Result<T> + Send + 'static,
) -> Result<T> {
    let token = credential(&state, &headers)?;
    let permit = state.slots.clone().try_acquire_owned().map_err(|_| {
        Error::ResourceConflict("request queue full; retry with the same command ID".into())
    })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let actor = state.hub.lookup_session(&token)?;
        f(&state.hub, &actor)
    })
    .await
    .map_err(|_| Error::StorageUnavailable("request worker stopped".into()))?
}

async fn bootstrap(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    let token = credential(&s, &h)?;
    if crate::digest::sha256_hex(token.as_bytes())
        != crate::digest::sha256_hex(s.bootstrap.as_bytes())
    {
        return Err(Error::AuthRequired);
    }
    let permit = s
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::ResourceConflict("request queue full".into()))?;
    let token = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        s.hub.ensure_session("owner")
    })
    .await
    .map_err(|_| Error::StorageUnavailable("session worker stopped".into()))??;
    Ok(Json(json!({"token": token})))
}

async fn session(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    call(s, h, |_, actor| Ok(Json(json!({"actor_id": actor})))).await
}

async fn logout(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    let token = credential(&s, &h)?;
    call(s, h, move |hub, _| {
        hub.revoke_session(&token)?;
        Ok(Json(json!({"revoked": true})))
    })
    .await
}

async fn doctor(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    call(s, h, |hub, _| Ok(Json(hub.doctor()))).await
}

async fn operation(
    State(s): State<AppState>,
    h: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>> {
    call(s, h, move |hub, actor| {
        Ok(Json(serde_json::to_value(hub.operation(actor, &id)?)?))
    })
    .await
}

async fn commands(
    State(s): State<AppState>,
    h: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>)> {
    if h.get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        != Some("application/json")
    {
        return Err(Error::InvalidContract(
            "Content-Type must be application/json".into(),
        ));
    }
    call(s, h, move |hub, actor| {
        Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::to_value(hub.command_bytes(actor, &body)?)?),
        ))
    })
    .await
}

async fn asset(uri: axum::http::Uri) -> Response {
    let name = if uri.path() == "/" {
        "index.html"
    } else {
        uri.path().trim_start_matches('/')
    };
    let Some((_, bytes)) = WEB_ASSETS.iter().find(|(key, _)| *key == name) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = if name.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if name.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if name.ends_with(".css") {
        "text/css; charset=utf-8"
    } else {
        "application/octet-stream"
    };
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header("Referrer-Policy", "no-referrer")
        .header("X-Content-Type-Options", "nosniff")
        .header(
            "Content-Security-Policy",
            "default-src 'self'; connect-src 'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none'",
        )
        .header("Cache-Control", "no-store")
        .body(Body::from(*bytes))
        .unwrap()
}
