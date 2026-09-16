use bf::digest::parse_strict_json;
use bf::domain::{cycle_in_ids, validate_task};
use bf::hub::Hub;
use serde_json::json;
use tempfile::tempdir;

fn hub() -> (tempfile::TempDir, Hub) {
    let dir = tempdir().unwrap();
    let hub = Hub::open(dir.path()).unwrap();
    (dir, hub)
}

#[test]
fn at001_unknown_dependency_is_not_completion() {
    let raw = include_str!("../fixtures/protocol/task.json");
    let mut v = parse_strict_json(raw).unwrap();
    v["depends_on"] = json!([{"id": "T-missing", "revision": 1}]);
    validate_task(&v).unwrap();
    assert!(!cycle_in_ids(&[("T-001".into(), "T-missing".into())]));
}

#[test]
fn at002_cycle_rejected() {
    assert!(cycle_in_ids(&[
        ("A".into(), "B".into()),
        ("B".into(), "A".into())
    ]));
}

#[test]
fn at003_command_replay_same_body() {
    let (_d, hub) = hub();
    let body = serde_json::to_vec(&json!({
        "schema_version": 3,
        "command_id": "CMD-1",
        "kind": "create_mission",
        "target_id": null,
        "expected_version": null,
        "payload": {"goal": "pr_ready"}
    }))
    .unwrap();
    let a = hub.command_bytes("owner-demo", &body).unwrap();
    let b = hub.command_bytes("owner-demo", &body).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(b.status, "replayed");
}

#[test]
fn at004_changed_body_conflicts() {
    let (_d, hub) = hub();
    let a = serde_json::to_vec(&json!({
        "schema_version": 3,
        "command_id": "CMD-2",
        "kind": "create_mission",
        "payload": {"goal": "pr_ready"}
    }))
    .unwrap();
    let b = serde_json::to_vec(&json!({
        "schema_version": 3,
        "command_id": "CMD-2",
        "kind": "create_mission",
        "payload": {"goal": "merged"}
    }))
    .unwrap();
    hub.command_bytes("owner-demo", &a).unwrap();
    let err = hub.command_bytes("owner-demo", &b).unwrap_err();
    assert_eq!(err.code(), "COMMAND_CONFLICT");
}

#[test]
fn at005_cf06_agent_cannot_take() {
    let (_d, hub) = hub();
    let body = serde_json::to_vec(&json!({
        "schema_version": 3,
        "command_id": "CMD-take",
        "kind": "take",
        "target_id": "T-001",
        "expected_version": 1,
        "payload": {"checkpoint_preference": "last_durable"}
    }))
    .unwrap();
    let err = hub.command_bytes("agent-demo", &body).unwrap_err();
    assert_eq!(err.code(), "POLICY_DENIED");
}

#[test]
fn hf17_agent_cannot_grant() {
    let (_d, hub) = hub();
    let body = serde_json::to_vec(&json!({
        "schema_version": 3,
        "command_id": "CMD-grant",
        "kind": "grant_allowance",
        "payload": {"amount": 1}
    }))
    .unwrap();
    let err = hub.command_bytes("agent-demo", &body).unwrap_err();
    assert_eq!(err.code(), "POLICY_DENIED");
}

#[test]
fn at012_truncated_frame() {
    let err = Hub::parse_result_frame(b"{\"schema_version\":3").unwrap_err();
    assert_eq!(err.code(), "INVALID_CONTRACT");
}

#[test]
fn duplicate_json_keys_rejected() {
    let err = parse_strict_json("{\"a\":1,\"a\":2}").unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn at032_gate_oracle() {
    let dir = tempdir().unwrap();
    let hub = Hub::open(dir.path()).unwrap();
    let work = dir.path().join("gate");
    let (buggy_fail, good_pass, wrong_fail) = hub.gate_at032(&work).unwrap();
    assert!(buggy_fail, "buggy fixture must fail the oracle");
    assert!(good_pass, "known-good must pass the oracle");
    assert!(
        wrong_fail,
        "injected defect must fail without removing the oracle"
    );
}

#[test]
fn demo_basic_reaches_review_ready() {
    let (_d, hub) = hub();
    let r = hub.run_fixture("basic").unwrap();
    assert_eq!(r.check_result, "pass");
    assert_eq!(r.phase, "review_ready");
    assert_eq!(r.author_authority, "sealed");
    assert_eq!(r.occupancy, "stopped");
    assert_eq!(r.effect_state, "confirmed");
    assert!(r.pr_number.is_some());
    assert!(r.fake);
    hub.writer_cannot_spend_completion().unwrap();
    hub.try_second_writer().unwrap();
}

#[test]
fn demo_interrupted_publish_reconciles_once() {
    let (_d, hub) = hub();
    let r = hub.run_fixture("interrupted_publish").unwrap();
    assert_eq!(r.check_result, "pass");
    assert_eq!(r.phase, "review_ready");
    assert_eq!(r.author_authority, "sealed");
    assert_eq!(r.occupancy, "stopped");
    assert_eq!(r.effect_state, "confirmed");
    assert_eq!(r.pr_number, Some(1));
}

#[test]
fn body_owner_does_not_authenticate() {
    let (_d, hub) = hub();
    let body = serde_json::to_vec(&json!({
        "schema_version": 3,
        "command_id": "CMD-owner",
        "kind": "create_mission",
        "payload": {"owner_id": "owner-demo", "goal": "pr_ready"}
    }))
    .unwrap();
    let err = hub.command_bytes("agent-demo", &body).unwrap_err();
    assert_eq!(err.code(), "POLICY_DENIED");
}
