//! Regression test for issue #162: enum-backed `*ExactMatch` wrappers must
//! serialize as bare scalars and must not swallow advanced filter objects when
//! used inside an untagged `*FilterProperty` enum (hook 17).

use camunda_orchestration_sdk::models::JobStateFilterProperty;

#[test]
fn exact_match_scalar_round_trips() {
    let payload = r#""CANCELED""#;
    let v: JobStateFilterProperty = serde_json::from_str(payload).expect("deserialize");
    assert!(
        matches!(v, JobStateFilterProperty::JobStateExactMatch(_)),
        "got {v:?}"
    );
    assert_eq!(serde_json::to_string(&v).unwrap(), payload);
}

#[test]
fn advanced_filter_object_round_trips() {
    let payload = r#"{"$eq":"CANCELED","$in":["CREATED","FAILED"]}"#;
    let v: JobStateFilterProperty = serde_json::from_str(payload).expect("deserialize");
    match &v {
        JobStateFilterProperty::AdvancedJobStateFilter(f) => {
            assert!(f.dollar_eq.is_some());
            assert_eq!(f.dollar_in.as_ref().map(Vec::len), Some(2));
        }
        other => panic!("advanced filter swallowed as {other:?}"),
    }
    let out: serde_json::Value = serde_json::to_value(&v).unwrap();
    let want: serde_json::Value = serde_json::from_str(payload).unwrap();
    assert_eq!(out, want);
}
