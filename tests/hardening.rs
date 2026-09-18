use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use bf::{Error, Hub};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::{tempdir, TempDir};
use tower::ServiceExt;
fn fresh() -> (TempDir, Arc<Hub>) {
    let dir = tempdir().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    (dir, hub)
}
fn cmd(
    h: &Hub,
    kind: &str,
    id: Option<&str>,
    version: Option<i64>,
    payload: Value,
) -> bf::Result<bf::Operation> {
    h.command_bytes("owner-demo",&serde_json::to_vec(&json!({"schema_version":3,"command_id":uuid::Uuid::new_v4().to_string(),"kind":kind,"target_id":id,"expected_version":version,"payload":payload})).unwrap())
}
fn propose(h: &Hub) -> Value {
    let op = cmd(
        h,
        "run",
        None,
        None,
        json!({"goal":"Reject repeats 🐈","project_id":"demo"}),
    )
    .unwrap();
    h.drive().unwrap();
    op.result
}
fn activate(h: &Hub) -> Value {
    let plan = propose(h);
    cmd(
        h,
        "start_work",
        plan["draft_id"].as_str(),
        Some(1),
        json!({}),
    )
    .unwrap()
    .result
}
fn sql(dir: &TempDir) -> Connection {
    Connection::open(dir.path().join("hub.sqlite")).unwrap()
}
async fn http(
    h: Arc<Hub>,
    token: Option<&str>,
    path: &str,
    body: Option<&str>,
    origin: Option<&str>,
) -> (StatusCode, Value) {
    let app = bf::api::router(bf::api::AppState::new(
        h,
        "http://127.0.0.1:7420".into(),
        "private-bootstrap".into(),
    ));
    let mut request = Request::builder()
        .uri(path)
        .header("host", "127.0.0.1:7420");
    if let Some(t) = token {
        request = request.header("authorization", format!("Bearer {t}"));
    }
    if let Some(o) = origin {
        request = request.header("origin", o);
    }
    if body.is_some() {
        request = request
            .method("POST")
            .header("content-type", "application/json");
    }
    let response = app
        .oneshot(
            request
                .body(Body::from(body.unwrap_or("").to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn http_denies_anonymous_and_cross_origin() {
    let (_d, h) = fresh();
    assert_eq!(
        http(h.clone(), None, "/v3/work", None, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        http(h.clone(), None, "/v3/doctor", None, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    let token = h.ensure_session("owner-demo").unwrap();
    assert_eq!(
        http(
            h,
            Some(&token),
            "/v3/work",
            None,
            Some("https://evil.invalid")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
async fn http_preserves_duplicate_keys_and_exact_raw_identity() {
    let (_d, h) = fresh();
    let token = h.ensure_session("owner-demo").unwrap();
    let bad = r#"{"schema_version":3,"command_id":"raw","kind":"create_mission","payload":{"goal":"pr_ready","goal":"merged"}}"#;
    assert_eq!(
        http(h.clone(), Some(&token), "/v3/commands", Some(bad), None)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let good = r#"{"schema_version":3,"command_id":"raw","kind":"create_mission","payload":{"goal":"pr_ready"}}"#;
    let a = http(h.clone(), Some(&token), "/v3/commands", Some(good), None).await;
    assert_eq!(a.0, StatusCode::ACCEPTED);
    let b = http(
        h.clone(),
        Some(&token),
        "/v3/commands",
        Some(&format!("{good} ")),
        None,
    )
    .await;
    assert_eq!(b.0, StatusCode::CONFLICT);
    let lookup = http(
        h,
        Some(&token),
        &format!("/v3/operations/{}", a.1["id"].as_str().unwrap()),
        None,
        None,
    )
    .await;
    assert_eq!(lookup.1["id"], a.1["id"]);
    assert_eq!(lookup.1["result"], a.1["result"]);
}
#[tokio::test]
async fn revoked_session_is_not_recovered_by_fallback() {
    let (_d, h) = fresh();
    let token = h.ensure_session("owner-demo").unwrap();
    h.revoke_session(&token).unwrap();
    assert_eq!(
        http(h, Some(&token), "/v3/work", None, None).await.0,
        StatusCode::UNAUTHORIZED
    );
}
#[test]
fn private_bootstrap_single_hub_and_fixed_sqlite() {
    let (d, h) = fresh();
    assert_eq!(h.sqlite_version().unwrap(), "3.53.2");
    assert!(matches!(
        Hub::open(d.path()),
        Err(Error::ResourceConflict(_))
    ));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(d.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
#[test]
fn replay_rechecks_current_principal_and_membership() {
    let (d, h) = fresh();
    let raw=br#"{"schema_version":3,"command_id":"a","kind":"create_mission","payload":{"goal":"pr_ready"}}"#;
    h.command_bytes("owner-demo", raw).unwrap();
    sql(&d)
        .execute(
            "DELETE FROM memberships WHERE principal_id='owner-demo' AND repo_id='repo-demo'",
            [],
        )
        .unwrap();
    assert!(h.command_bytes("owner-demo", raw).is_err());
    drop(h);
    let reopened = Hub::open(d.path()).unwrap();
    assert!(reopened.command_bytes("owner-demo", raw).is_err());
}
#[test]
fn cross_owner_target_and_operations_are_denied() {
    let (d, h) = fresh();
    let op = cmd(&h, "create_mission", None, None, json!({"goal":"pr_ready"})).unwrap();
    sql(&d).execute_batch("INSERT INTO principals VALUES('other','human',1);INSERT INTO memberships VALUES('other','repo-demo','administrator');").unwrap();
    let body = json!({"schema_version":3,"command_id":"other","kind":"stop","target_id":op.result["mission_id"],"expected_version":1,"payload":{}});
    assert!(h
        .command_bytes("other", &serde_json::to_vec(&body).unwrap())
        .is_err());
    assert!(h.operation("other", &op.id).is_err());
    assert!(h.work_for("other", None, 50).unwrap().is_empty());
}
#[test]
fn draft_edit_revision_and_single_acceptance() {
    let (_d, h) = fresh();
    let plan = propose(&h);
    let id = plan["draft_id"].as_str();
    let edit = cmd(
        &h,
        "edit_draft",
        id,
        Some(1),
        json!({"goal":"Unicode 🐈\nsecond line","project_id":"demo"}),
    )
    .unwrap();
    assert_eq!(edit.result["revision"], 2);
    assert!(matches!(
        cmd(&h, "start_work", id, Some(1), json!({})),
        Err(Error::StaleVersion)
    ));
    h.drive().unwrap();
    cmd(&h, "start_work", id, Some(2), json!({})).unwrap();
    assert!(cmd(&h, "start_work", id, Some(2), json!({})).is_err());
    h.drive().unwrap();
    assert_eq!(h.work().unwrap().len(), 1);
    assert_eq!(h.work().unwrap()[0].phase, "review_ready");
}
#[test]
fn two_fixture_missions_do_not_conflict() {
    let (_d, h) = fresh();
    activate(&h);
    h.drive().unwrap();
    activate(&h);
    h.drive().unwrap();
    assert_eq!(h.work().unwrap().len(), 2);
    assert!(h.work().unwrap().iter().all(|w| w.phase == "review_ready"));
}
#[test]
fn revoked_expired_exhausted_grants_refuse_before_job_intent() {
    for change in [
        "revoked=1",
        "expires_at='2000-01-01T00:00:00Z'",
        "used_invocations=invocation_allowance",
    ] {
        let (d, h) = fresh();
        sql(&d)
            .execute(&format!("UPDATE grants SET {change}"), [])
            .unwrap();
        assert!(cmd(
            &h,
            "run",
            None,
            None,
            json!({"goal":"x","project_id":"demo"})
        )
        .is_err());
        let count: i64 = sql(&d)
            .query_row("SELECT count(*) FROM jobs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
#[test]
fn failed_storage_rolls_back_mission_event_and_operation() {
    let (d, h) = fresh();
    sql(&d).execute_batch("CREATE TRIGGER fail_command BEFORE INSERT ON commands BEGIN SELECT RAISE(ABORT,'disk failure fixture'); END;").unwrap();
    assert!(cmd(&h, "create_mission", None, None, json!({"goal":"pr_ready"})).is_err());
    for table in ["missions", "commands", "operations", "events"] {
        let count: i64 = sql(&d)
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
#[test]
fn stop_before_dispatch_releases_holds_and_rejects_stale_control() {
    let (d, h) = fresh();
    let op = cmd(
        &h,
        "run",
        None,
        None,
        json!({"goal":"x","project_id":"demo"}),
    )
    .unwrap();
    let mission = op.result["mission_id"].as_str();
    let stopped = cmd(&h, "stop", mission, Some(1), json!({})).unwrap();
    assert_eq!(stopped.result["status"], "stopped");
    assert!(cmd(&h, "stop", mission, Some(1), json!({})).is_err());
    let held: i64 = sql(&d)
        .query_row(
            "SELECT count(*) FROM budget_holds WHERE state='held'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(held, 0);
    h.drive().unwrap();
    assert_eq!(h.occupied_slots().unwrap(), 0);
}
#[test]
fn cancel_cannot_resume_or_publish() {
    let (_d, h) = fresh();
    let op = activate(&h);
    h.set_fixture_fault("before_dispatch").unwrap();
    assert!(h.drive().is_err());
    cmd(&h, "cancel", op["mission_id"].as_str(), Some(1), json!({})).unwrap();
    assert!(cmd(&h, "resume", op["mission_id"].as_str(), Some(2), json!({})).is_err());
    h.reconcile_effects().unwrap();
    assert_eq!(h.work().unwrap()[0].phase, "cancelled");
    assert!(h.work().unwrap()[0].pr.is_none());
}
#[test]
fn publication_recovers_each_durable_boundary_in_fresh_hub() {
    for point in [
        "before_dispatch",
        "after_dispatch",
        "after_remote",
        "after_receipt",
        "before_settlement",
    ] {
        let (d, h) = fresh();
        let op = activate(&h);
        h.set_fixture_fault(point).unwrap();
        assert!(
            matches!(h.drive(), Err(Error::OutcomeUnknown(_))),
            "{point}"
        );
        drop(h);
        let reopened = Hub::open(d.path()).unwrap();
        let receipt = reopened
            .receipt("fault", op["task_id"].as_str().unwrap())
            .unwrap();
        assert_eq!(receipt.phase, "review_ready", "{point}");
        assert_eq!(receipt.pr_number, Some(1));
        let forge = bf::forge::FakeForge::open(&d.path().join("fake-forge")).unwrap();
        assert!(forge
            .find(&format!("pr:{}:1", receipt.task_id))
            .unwrap()
            .is_some());
    }
}
#[test]
fn remote_observation_survives_revocation_without_readiness() {
    let (d, h) = fresh();
    activate(&h);
    h.set_fixture_fault("after_remote").unwrap();
    assert!(h.drive().is_err());
    h.set_grant_revoked(true).unwrap();
    drop(h);
    let reopened = Hub::open(d.path()).unwrap();
    assert_eq!(reopened.work().unwrap()[0].phase, "checking");
    let state: String = sql(&d)
        .query_row("SELECT state FROM effects", [], |r| r.get(0))
        .unwrap();
    assert_eq!(state, "confirmed");
}
#[test]
fn human_closed_remote_is_never_reopened() {
    let (d, h) = fresh();
    let op = activate(&h);
    h.set_fixture_fault("after_remote").unwrap();
    assert!(h.drive().is_err());
    bf::forge::FakeForge::open(&d.path().join("fake-forge"))
        .unwrap()
        .mark_human_closed(&format!("pr:{}:1", op["task_id"].as_str().unwrap()))
        .unwrap();
    drop(h);
    let _reopened = Hub::open(d.path()).unwrap();
    let state: String = sql(&d)
        .query_row("SELECT state FROM effects", [], |r| r.get(0))
        .unwrap();
    assert_eq!(state, "blocked");
}
#[test]
fn repeated_demo_preserves_source_objects_and_artifacts() {
    let (d, h) = fresh();
    let first = h.run_fixture("basic").unwrap();
    let artifacts = std::fs::read_dir(d.path().join("artifacts"))
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            let bytes = std::fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect::<Vec<_>>();
    let source = d
        .path()
        .join("repos")
        .join(&first.mission_id)
        .join("plan-1");
    let head = bf::gitutil::git(&source, &["rev-parse", "HEAD"]).unwrap();
    drop(h);
    let reopened = Hub::open(d.path()).unwrap();
    let second = reopened.run_fixture("basic").unwrap();
    assert_eq!(first.candidate_id, second.candidate_id);
    assert_eq!(
        head,
        bf::gitutil::git(&source, &["rev-parse", "HEAD"]).unwrap()
    );
    for (p, b) in artifacts {
        assert_eq!(std::fs::read(p).unwrap(), b);
    }
}
#[test]
fn early_exit_and_forged_completion_cannot_pass() {
    let (d, h) = fresh();
    let root = d.path().join("hostile");
    std::fs::create_dir_all(root.join("src")).unwrap();
    for source in [
        "raise SystemExit(0)",
        "import os; os._exit(0)",
        "print('PASS'); raise SystemExit(0)",
        "print('{\"complete\":true}'); raise SystemExit(0)",
    ] {
        std::fs::write(root.join("src/dedup.py"), source).unwrap();
        assert!(h.verify_fixture_source(&root).is_err());
    }
    std::fs::remove_file(root.join("src/dedup.py")).unwrap();
    assert!(h.verify_fixture_source(&root).is_err());
}
#[test]
fn migration_preserves_old_records_and_does_not_enable_live() {
    let d = tempdir().unwrap();
    {
        let c = sql(&d);
        c.execute_batch(include_str!("../migrations/001_core.sql"))
            .unwrap();
        c.execute_batch("INSERT INTO principals VALUES('owner-demo','human',1);INSERT INTO repositories VALUES('repo-demo','fake','{}');INSERT INTO memberships VALUES('owner-demo','repo-demo','administrator');INSERT INTO domains VALUES('core','owner-demo','{}');INSERT INTO grants VALUES('demo-grant','owner-demo','owner-demo','2099-01-01T00:00:00Z',0,10,0,'{\"implement\":true}');INSERT INTO sessions VALUES('old-token','owner-demo','2026-01-01');INSERT INTO missions VALUES('old','owner-demo','core','pr_ready',1,0,'{}');").unwrap();
    }
    let h = Hub::open(d.path()).unwrap();
    assert!(h.lookup_session("old-token").is_err());
    let c = sql(&d);
    let old: i64 = c
        .query_row("SELECT count(*) FROM missions WHERE id='old'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(old, 1);
    let fixture: bool = c
        .query_row(
            "SELECT fixture_only AND json_extract(capability_json,'$.plan') FROM grants",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(fixture);
}
#[test]
fn restart_keeps_unknown_occupancy_and_cost_reserved() {
    let (d, h) = fresh();
    cmd(
        &h,
        "run",
        None,
        None,
        json!({"goal":"x","project_id":"demo"}),
    )
    .unwrap();
    sql(&d)
        .execute("UPDATE jobs SET lifecycle='starting'", [])
        .unwrap();
    drop(h);
    let h = Hub::open(d.path()).unwrap();
    assert_eq!(h.occupied_slots().unwrap(), 1);
    let c = sql(&d);
    let lifecycle: String = c
        .query_row("SELECT lifecycle FROM jobs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(lifecycle, "unknown");
    let budget: String = c
        .query_row("SELECT state FROM budget_holds", [], |r| r.get(0))
        .unwrap();
    assert_eq!(budget, "uncertain");
}
#[test]
fn wrong_producer_and_stale_evidence_are_rejected() {
    let (d, h) = fresh();
    h.run_fixture("basic").unwrap();
    let c = sql(&d);
    let insert="INSERT INTO evidence SELECT ?1,job_id,candidate_id,subject_digest,?2,check_id,result,receipt_json FROM evidence LIMIT 1";
    assert!(c.execute(insert, params!["forged", "owner-demo"]).is_err());
    assert!(c
        .execute(insert, params!["stale", "verifier-fixture"])
        .is_err());
}
#[test]
fn concurrent_duplicate_submissions_activate_once() {
    let (_d, h) = fresh();
    let raw=serde_json::to_vec(&json!({"schema_version":3,"command_id":"concurrent","kind":"run","payload":{"goal":"x","project_id":"demo"}})).unwrap();
    let threads = (0..8)
        .map(|_| {
            let h = h.clone();
            let raw = raw.clone();
            std::thread::spawn(move || h.command_bytes("owner-demo", &raw).unwrap().id)
        })
        .collect::<Vec<_>>();
    let ids = threads
        .into_iter()
        .map(|t| t.join().unwrap())
        .collect::<Vec<_>>();
    assert!(ids.iter().all(|id| id == &ids[0]));
}

#[test]
fn observer_cannot_activate_or_dispatch_task_repository() {
    let (d, h) = fresh();
    let plan = propose(&h);
    sql(&d)
        .execute(
            "UPDATE memberships SET role='observer' WHERE repo_id LIKE 'fixture-%'",
            [],
        )
        .unwrap();
    assert!(cmd(
        &h,
        "start_work",
        plan["draft_id"].as_str(),
        Some(1),
        json!({})
    )
    .is_err());
    sql(&d)
        .execute(
            "UPDATE memberships SET role='administrator' WHERE repo_id LIKE 'fixture-%'",
            [],
        )
        .unwrap();
    cmd(
        &h,
        "start_work",
        plan["draft_id"].as_str(),
        Some(1),
        json!({}),
    )
    .unwrap();
    sql(&d)
        .execute(
            "UPDATE memberships SET role='observer' WHERE repo_id LIKE 'fixture-%'",
            [],
        )
        .unwrap();
    h.drive().unwrap();
    let spawned:i64=sql(&d).query_row("SELECT count(*) FROM jobs WHERE purpose='implement' AND lifecycle IN('running','succeeded')",[],|r|r.get(0)).unwrap();
    assert_eq!(spawned, 0);
}
#[test]
fn stopped_prepared_work_can_retry_within_cumulative_limits() {
    let (d, h) = fresh();
    let op = activate(&h);
    let mission = op["mission_id"].as_str();
    cmd(&h, "stop", mission, Some(1), json!({})).unwrap();
    assert_eq!(h.work().unwrap()[0].phase, "failed");
    cmd(&h, "resume", mission, Some(2), json!({})).unwrap();
    let task = h.work().unwrap().remove(0);
    let retry = cmd(
        &h,
        "retry_task",
        Some(&task.task_id),
        Some(task.version),
        json!({}),
    );
    assert!(retry.is_ok(), "{retry:?}");
    h.drive().unwrap();
    assert_eq!(h.work().unwrap()[0].phase, "review_ready");
    let invocations: i64 = sql(&d)
        .query_row("SELECT lifetime_invocations FROM tasks", [], |r| r.get(0))
        .unwrap();
    assert_eq!(invocations, 2);
}
#[test]
fn expired_queue_entry_does_not_starve_later_planner() {
    let (d, h) = fresh();
    cmd(
        &h,
        "run",
        None,
        None,
        json!({"goal":"first","project_id":"demo"}),
    )
    .unwrap();
    sql(&d)
        .execute("UPDATE jobs SET deadline_at='2000-01-01T00:00:00Z'", [])
        .unwrap();
    cmd(
        &h,
        "run",
        None,
        None,
        json!({"goal":"second","project_id":"demo"}),
    )
    .unwrap();
    h.drive().unwrap();
    h.drive().unwrap();
    let complete: i64 = sql(&d)
        .query_row(
            "SELECT count(*) FROM jobs WHERE lifecycle='succeeded'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(complete, 1);
}
#[test]
fn missing_artifact_blocks_publication_after_pass() {
    let (d, h) = fresh();
    activate(&h);
    h.set_fixture_fault("before_dispatch").unwrap();
    assert!(h.drive().is_err());
    for entry in std::fs::read_dir(d.path().join("artifacts")).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    h.reconcile_effects().unwrap();
    let state: String = sql(&d)
        .query_row("SELECT state FROM effects", [], |r| r.get(0))
        .unwrap();
    assert_eq!(state, "blocked");
}
#[tokio::test]
async fn slow_streams_have_a_lifetime_bound_and_control_remains_available() {
    let (_d, h) = fresh();
    let token = h.ensure_session("owner-demo").unwrap();
    let app = bf::api::router(bf::api::AppState::new(
        h,
        "http://127.0.0.1:7420".into(),
        "bootstrap".into(),
    ));
    let mut held = Vec::new();
    for _ in 0..16 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v3/events")
                    .header("host", "127.0.0.1:7420")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        held.push(response);
    }
    let extra = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v3/events")
                .header("host", "127.0.0.1:7420")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(extra.status(), StatusCode::CONFLICT);
    let work = app
        .oneshot(
            Request::builder()
                .uri("/v3/work")
                .header("host", "127.0.0.1:7420")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(work.status(), StatusCode::OK);
    drop(held);
}

#[test]
fn stopped_prepared_verifier_can_retry_and_preserves_candidate() {
    let (d, h) = fresh();
    let op = activate(&h);
    sql(&d).execute_batch("CREATE TRIGGER pause_before_verifier AFTER INSERT ON jobs WHEN NEW.purpose='verify' BEGIN UPDATE missions SET paused=1 WHERE id=NEW.mission_id; END;").unwrap();
    h.drive().unwrap();
    assert_eq!(h.work().unwrap()[0].phase, "checking");
    let mission = op["mission_id"].as_str();
    cmd(&h, "stop", mission, Some(1), json!({})).unwrap();
    assert_eq!(h.work().unwrap()[0].phase, "failed");
    sql(&d)
        .execute_batch("DROP TRIGGER pause_before_verifier")
        .unwrap();
    cmd(&h, "resume", mission, Some(2), json!({})).unwrap();
    let task = h.work().unwrap().remove(0);
    cmd(
        &h,
        "retry_task",
        Some(&task.task_id),
        Some(task.version),
        json!({}),
    )
    .unwrap();
    h.drive().unwrap();
    assert_eq!(h.work().unwrap()[0].phase, "review_ready");
    let candidates: i64 = sql(&d)
        .query_row("SELECT count(*) FROM candidates", [], |r| r.get(0))
        .unwrap();
    assert_eq!(candidates, 2);
}

#[test]
fn rejected_planner_advances_owner_event_cursor() {
    let (d, h) = fresh();
    cmd(
        &h,
        "run",
        None,
        None,
        json!({"goal":"expired planner","project_id":"demo"}),
    )
    .unwrap();
    let before = h.change_cursor("owner-demo").unwrap();
    sql(&d)
        .execute("UPDATE jobs SET deadline_at='2000-01-01T00:00:00Z'", [])
        .unwrap();
    h.drive().unwrap();
    assert!(h.change_cursor("owner-demo").unwrap() > before);
    let failed: i64 = sql(&d)
        .query_row(
            "SELECT count(*) FROM drafts WHERE status='failed'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(failed, 1);
}
