//! Regression test for issue #41: every polymorphic (`oneOf` + `discriminator`)
//! type in the generated client must round-trip through its serde
//! internally-tagged enum.
//!
//! `openapi-generator` emitted a `#[serde(tag = "...")]` enum *and* left the
//! discriminator declared as an ordinary field on each variant struct. Under
//! serde's internally-tagged representation the tag is consumed by the enum and
//! not forwarded to the variant, so deserialization failed with
//! "missing field `<tag>`", and serialization wrote the tag twice (a duplicate
//! JSON key). `hook_12` strips the re-declared field; this test guards every one
//! of the six polymorphic types against a representative wire payload, in both
//! directions, so a regeneration that reintroduced the defect would fail here.

use camunda_orchestration_sdk::models::{
    AgentInstanceMessageContent, AncestorScopeInstruction, ClusterRestoreOperation, JobResult,
    SourceElementInstruction, WaitStateDetails,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

/// Deserialize `payload` into `T`, serialize it back, and assert:
///   * it deserializes at all (the pre-fix failure was "missing field `<tag>`"),
///   * the re-serialized value is byte-for-byte the original payload, and
///   * the discriminator key appears **exactly once** in the output (the pre-fix
///     failure emitted it twice as a duplicate JSON key).
fn assert_roundtrips<T>(payload: Value, tag: &str)
where
    T: DeserializeOwned + Serialize,
{
    let decoded: T = serde_json::from_value(payload.clone())
        .unwrap_or_else(|e| panic!("deserialize failed for tag `{tag}`: {e}"));
    let reserialized = serde_json::to_value(&decoded).expect("serialize");
    assert_eq!(reserialized, payload, "round-trip mismatch for tag `{tag}`",);
    let text = serde_json::to_string(&decoded).expect("serialize to string");
    let occurrences = text.matches(&format!("\"{tag}\"")).count();
    assert_eq!(
        occurrences, 1,
        "discriminator `{tag}` must be written exactly once, got: {text}",
    );
}

#[test]
fn job_result_round_trips() {
    // Request-side, core worker completion path.
    assert_roundtrips::<JobResult>(json!({ "type": "userTask" }), "type");
}

#[test]
fn wait_state_details_round_trips() {
    // Response-side, process-instance queries.
    assert_roundtrips::<WaitStateDetails>(
        json!({ "waitStateType": "SIGNAL", "signalName": "order-received" }),
        "waitStateType",
    );
}

#[test]
fn ancestor_scope_instruction_round_trips() {
    // Variant whose only property was the discriminator — stripping empties it.
    assert_roundtrips::<AncestorScopeInstruction>(
        json!({ "ancestorScopeType": "sourceParent" }),
        "ancestorScopeType",
    );
}

#[test]
fn source_element_instruction_round_trips() {
    assert_roundtrips::<SourceElementInstruction>(
        json!({ "sourceType": "byId", "sourceElementId": "approve-task" }),
        "sourceType",
    );
}

#[test]
fn agent_instance_message_content_round_trips() {
    assert_roundtrips::<AgentInstanceMessageContent>(
        json!({ "contentType": "OBJECT", "object": { "answer": 42 } }),
        "contentType",
    );
}

#[test]
fn cluster_restore_operation_round_trips() {
    assert_roundtrips::<ClusterRestoreOperation>(
        json!({ "operation": "UpdateIncarnationNumberOperation", "brokerId": "0" }),
        "operation",
    );
}
