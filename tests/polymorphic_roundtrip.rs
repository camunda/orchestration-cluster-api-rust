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

/// Deserialize the raw JSON `payload` into `T`, serialize it back to a string, and assert:
///   * it deserializes at all (the pre-fix failure was "missing field `<tag>`"),
///   * the re-serialized JSON is **byte-for-byte** the original `payload`, and
///   * the discriminator key appears **exactly once** in the output (the pre-fix
///     failure emitted it twice as a duplicate JSON key).
///
/// `payload` is kept as a raw `&str` — deliberately *not* a `serde_json::Value` — so the
/// comparison is genuinely byte-exact: parsing to a `Value` first would normalise key
/// order and silently collapse a duplicate key (`Value` cannot even represent one),
/// hiding the very duplicate-tag defect this test guards. Each fixture is therefore
/// written in the exact compact form serde emits: the internally-tagged discriminator
/// first, then the variant's fields in declaration order.
fn assert_roundtrips<T>(payload: &str, tag: &str)
where
    T: DeserializeOwned + Serialize,
{
    let decoded: T = serde_json::from_str(payload)
        .unwrap_or_else(|e| panic!("deserialize failed for tag `{tag}`: {e}"));
    let reserialized = serde_json::to_string(&decoded).expect("serialize");
    assert_eq!(reserialized, payload, "round-trip mismatch for tag `{tag}`");
    let occurrences = reserialized.matches(&format!("\"{tag}\"")).count();
    assert_eq!(
        occurrences, 1,
        "discriminator `{tag}` must be written exactly once, got: {reserialized}",
    );
}

#[test]
fn job_result_round_trips() {
    // Request-side, core worker completion path.
    //
    // NB: `JobResult`'s variant discriminators were re-declared as
    // `Option<String>` + `skip_serializing_if = "Option::is_none"` (unlike the
    // response-side unions below, whose discriminator was a *required* field).
    // For that optional shape there is no observable runtime defect to guard: serde
    // consumes the tag for the enum, the variant's optional field stays `None`, and
    // `skip_serializing_if` omits it — so a payload of `{"type":"userTask"}`
    // round-trips byte-for-byte to a single `type` key *whether or not* the field is
    // re-declared. The pre-fix duplicate-key / "missing field" failures only manifest
    // for the required-discriminator unions asserted below. Reintroduction of the
    // redundant optional field on `JobResult` is therefore caught structurally by the
    // generation-time guard (`scripts/test_hooks.py::NoVariantRedeclaresItsTagTest`),
    // not by this runtime round-trip. This case still pins that the request-side
    // path deserializes and emits exactly one tag.
    assert_roundtrips::<JobResult>(r#"{"type":"userTask"}"#, "type");
}

#[test]
fn wait_state_details_round_trips() {
    // Response-side, process-instance queries.
    assert_roundtrips::<WaitStateDetails>(
        r#"{"waitStateType":"SIGNAL","signalName":"order-received"}"#,
        "waitStateType",
    );
}

#[test]
fn ancestor_scope_instruction_round_trips() {
    // Variant whose only property was the discriminator — stripping empties it.
    assert_roundtrips::<AncestorScopeInstruction>(
        r#"{"ancestorScopeType":"sourceParent"}"#,
        "ancestorScopeType",
    );
}

#[test]
fn source_element_instruction_round_trips() {
    assert_roundtrips::<SourceElementInstruction>(
        r#"{"sourceType":"byId","sourceElementId":"approve-task"}"#,
        "sourceType",
    );
}

#[test]
fn agent_instance_message_content_round_trips() {
    assert_roundtrips::<AgentInstanceMessageContent>(
        r#"{"contentType":"OBJECT","object":{"answer":42}}"#,
        "contentType",
    );
}

#[test]
fn cluster_restore_operation_round_trips() {
    assert_roundtrips::<ClusterRestoreOperation>(
        r#"{"operation":"UpdateIncarnationNumberOperation","brokerId":"0"}"#,
        "operation",
    );
}
