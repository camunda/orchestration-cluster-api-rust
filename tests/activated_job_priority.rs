//! Regression test for issue #2:
//! `ActivatedJobResult.priority` must tolerate an absent value.
//!
//! The upstream `main` spec marks `priority` required, but released and alpha
//! servers (8.9.x, 8.10.0-alpha*) omit it from `POST /v2/jobs/activation`
//! responses. Without `#[serde(default)]` this made *every* activation with at
//! least one job fail to deserialize with "missing field `priority`", silently
//! stalling job workers. Hook 08 (version-skew-tolerance) applies the default;
//! this test guards the behavior against regeneration drift.

use camunda_orchestration_sdk::models::ActivatedJobResult;

/// A realistic activation-response job object. `priority` is intentionally omitted
/// to mirror what released servers actually return.
fn job_json_without_priority() -> serde_json::Value {
    serde_json::json!({
        "type": "greet",
        "processDefinitionId": "greet-process",
        "processDefinitionVersion": 1,
        "elementId": "greet-task",
        "customHeaders": {},
        "worker": "test-worker",
        "retries": 3,
        "deadline": 1_700_000_000_000i64,
        "variables": {},
        "tenantId": "<default>",
        "jobKey": "2251799813685250",
        "processInstanceKey": "2251799813685249",
        "processDefinitionKey": "2251799813685248",
        "elementInstanceKey": "2251799813685251",
        "kind": "BPMN_ELEMENT",
        "listenerEventType": "UNSPECIFIED",
        "userTask": null,
        "tags": [],
        "rootProcessInstanceKey": null
        // note: no "priority" key
    })
}

#[test]
fn deserializes_job_when_priority_is_absent() {
    let value = job_json_without_priority();
    let job: ActivatedJobResult =
        serde_json::from_value(value).expect("activation job without `priority` must deserialize");
    assert_eq!(job.priority, 0, "absent priority must default to 0");
}

#[test]
fn deserializes_job_when_priority_is_present() {
    let mut value = job_json_without_priority();
    value["priority"] = serde_json::json!(42);
    let job: ActivatedJobResult =
        serde_json::from_value(value).expect("activation job with `priority` must deserialize");
    assert_eq!(job.priority, 42, "present priority must be read through");
}
