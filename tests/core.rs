use axum::body::Body;
use axum::http::{Request, StatusCode};
use bf::api::{router, AppState};
use bf::Hub;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

#[test]
fn second_hub_is_refused() {
    let dir = TempDir::new().unwrap();
    let first = Hub::open(dir.path()).unwrap();
    let err = Hub::open(dir.path()).err().expect("second hub");
    assert_eq!(err.code(), "RESOURCE_CONFLICT");
    drop(first);
}

#[test]
fn doctor_reports_pinned_sqlite() {
    let dir = TempDir::new().unwrap();
    let hub = Hub::open(dir.path()).unwrap();
    let doctor = hub.doctor();
    assert_eq!(doctor["ok"], true);
    assert_eq!(doctor["sqlite"], "3.53.2");
}

#[tokio::test]
async fn http_auth_and_command_replay() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let origin = "http://127.0.0.1:9";
    let bootstrap = "boot-secret";
    let app = router(AppState::new(hub, origin.into(), bootstrap.into()));
    let health = app
        .clone()
        .oneshot(
            Request::get("http://127.0.0.1:9/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let denied = app
        .clone()
        .oneshot(
            Request::get("http://127.0.0.1:9/v3/doctor")
                .header("host", "127.0.0.1:9")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

    let login = app
        .clone()
        .oneshot(
            Request::post("http://127.0.0.1:9/v3/bootstrap")
                .header("host", "127.0.0.1:9")
                .header("authorization", "Bearer boot-secret")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let token =
        serde_json::from_slice::<Value>(&login.into_body().collect().await.unwrap().to_bytes())
            .unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();

    let body = json!({"schema_version":3,"command_id":"n1","kind":"note","payload":{"text":"hi"}});
    let raw = serde_json::to_vec(&body).unwrap();
    let first = app
        .clone()
        .oneshot(
            Request::post("http://127.0.0.1:9/v3/commands")
                .header("host", "127.0.0.1:9")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(raw.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::ACCEPTED);
    let one: Value =
        serde_json::from_slice(&first.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let second = app
        .oneshot(
            Request::post("http://127.0.0.1:9/v3/commands")
                .header("host", "127.0.0.1:9")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(raw))
                .unwrap(),
        )
        .await
        .unwrap();
    let two: Value =
        serde_json::from_slice(&second.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(one["id"], two["id"]);
}
