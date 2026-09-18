use crate::{Error, Hub, Result};
use axum::{
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
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
    streams: Arc<Semaphore>,
}
impl AppState {
    pub fn new(hub: Arc<Hub>, origin: String, bootstrap: String) -> Self {
        Self {
            hub,
            origin,
            bootstrap,
            slots: Arc::new(Semaphore::new(32)),
            streams: Arc::new(Semaphore::new(16)),
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
        .route("/v3/work", get(work))
        .route("/v3/projects", get(projects))
        .route("/v3/drafts", get(drafts))
        .route("/v3/drafts/{id}", get(detail))
        .route("/v3/commands", post(commands))
        .route("/v3/operations/{id}", get(operation))
        .route("/v3/events", get(events))
        .fallback(get(asset))
        .layer(DefaultBodyLimit::max(65536))
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
        s.hub.ensure_session("owner-demo")
    })
    .await
    .map_err(|_| Error::StorageUnavailable("session worker stopped".into()))??;
    Ok(Json(json!({"token":token})))
}
async fn session(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    call(s, h, |_, actor| Ok(Json(json!({"actor_id":actor})))).await
}
async fn logout(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    let token = credential(&s, &h)?;
    call(s, h, move |hub, _| {
        hub.revoke_session(&token)?;
        Ok(Json(json!({"revoked":true})))
    })
    .await
}
async fn doctor(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    call(s, h, |hub, _| Ok(Json(hub.doctor()))).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Page {
    before: Option<i64>,
    limit: Option<usize>,
}
async fn work(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(p): Query<Page>,
) -> Result<Json<Value>> {
    call(s, h, move |hub, actor| {
        let items = hub.work_for(actor, p.before, p.limit.unwrap_or(50))?;
        Ok(Json(
            json!({"next_cursor":items.last().map(|i|i.cursor),"items":items}),
        ))
    })
    .await
}
async fn projects(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    call(s, h, |hub, actor| Ok(Json(hub.projects(actor)?))).await
}
async fn drafts(State(s): State<AppState>, h: HeaderMap) -> Result<Json<Value>> {
    call(s, h, |hub, actor| Ok(Json(hub.drafts(actor)?))).await
}
async fn detail(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    call(s, h, move |hub, actor| Ok(Json(hub.detail(actor, &id)?))).await
}
async fn operation(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<String>,
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
    // The original bytes reach the strict decoder unchanged; session is rechecked in the command transaction.
    let token = credential(&s, &h)?;
    call(s, h, move |hub, _| {
        Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::to_value(hub.command_session(&token, &body)?)?),
        ))
    })
    .await
}
async fn events(State(s): State<AppState>, h: HeaderMap) -> Result<Response> {
    call(s.clone(), h.clone(), |_, _| Ok(())).await?;
    let permit = s
        .streams
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::ResourceConflict("event stream limit reached".into()))?;
    let stream = async_stream::stream! {
        let _permit=permit;
        let mut interval=tokio::time::interval(std::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let snapshot=call(s.clone(),h.clone(),|hub,actor|Ok(json!({"cursor":hub.change_cursor(actor)?}))).await;
            match snapshot {
                Ok(value)=>yield Ok::<_,std::convert::Infallible>(Event::default().event("snapshot").data(value.to_string())),
                Err(error)=>{yield Ok(Event::default().event("disconnected").data(error.code()));break;}
            }
        }
    };
    // Pull-based latest snapshots: each slow client holds at most one bounded page, no event queue.
    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response())
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
    Response::builder().header(header::CONTENT_TYPE,mime).header("Referrer-Policy","no-referrer").header("X-Content-Type-Options","nosniff").header("Content-Security-Policy","default-src 'self'; connect-src 'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none'").header("Cache-Control","no-store").body(Body::from(*bytes)).unwrap()
}
