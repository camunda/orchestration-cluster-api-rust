//! Clearable date-time fields: the API clears a user task's `dueDate` / `followUpDate`
//! with an empty string, while `null` or omitting the property preserves the value.
//!
//! The generated models type these fields `Option<Option<DateTime<FixedOffset>>>` and
//! `hook_19` gives the three states their wire forms: `None` is omitted (preserve),
//! `Some(Some(date))` is RFC 3339 (set) and `Some(None)` is `""` (clear). Every clearable
//! field on every model that has one is covered, in both directions, so a regeneration
//! that loses the hook (or turns a field into an empty `ClearableDateTime` struct) fails
//! here.

use camunda_orchestration_sdk::models::{
    Changeset, JobResult, JobResultCorrections, JobResultUserTask, UserTaskUpdateRequest,
};
use camunda_orchestration_sdk::{CompletionResult, UserTaskCorrections, UserTaskResult};
use chrono::{DateTime, FixedOffset};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;

type Clearable = Option<Option<DateTime<FixedOffset>>>;

fn date() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-12-01T09:00:00+02:00").unwrap()
}

/// A model with its clearable fields exposed by wire name, so each case runs against
/// every field of every model.
trait ClearableModel: Default + Serialize + DeserializeOwned + std::fmt::Debug {
    fn field(&mut self, wire: &str) -> &mut Clearable;
    fn clear(&mut self, wire: &str);
    fn is_cleared(&self, wire: &str) -> bool;
}

macro_rules! clearable_model {
    ($model:ty) => {
        impl ClearableModel for $model {
            fn field(&mut self, wire: &str) -> &mut Clearable {
                match wire {
                    "dueDate" => &mut self.due_date,
                    "followUpDate" => &mut self.follow_up_date,
                    other => panic!("not a clearable field: {other}"),
                }
            }
            fn clear(&mut self, wire: &str) {
                match wire {
                    "dueDate" => self.clear_due_date(),
                    "followUpDate" => self.clear_follow_up_date(),
                    other => panic!("not a clearable field: {other}"),
                }
            }
            fn is_cleared(&self, wire: &str) -> bool {
                match wire {
                    "dueDate" => self.is_due_date_cleared(),
                    "followUpDate" => self.is_follow_up_date_cleared(),
                    other => panic!("not a clearable field: {other}"),
                }
            }
        }
    };
}

clearable_model!(JobResultCorrections);
clearable_model!(Changeset);

const FIELDS: [&str; 2] = ["dueDate", "followUpDate"];

fn serializes_each_state<M: ClearableModel>() {
    for wire in FIELDS {
        let mut preserved = M::default();
        *preserved.field(wire) = None;
        assert_eq!(
            serde_json::to_value(&preserved).unwrap(),
            json!({}),
            "{wire}"
        );
        assert!(!preserved.is_cleared(wire), "{wire}");

        let mut set = M::default();
        *set.field(wire) = Some(Some(date()));
        assert_eq!(
            serde_json::to_value(&set).unwrap(),
            json!({ wire: "2026-12-01T09:00:00+02:00" }),
            "{wire}"
        );
        assert!(!set.is_cleared(wire), "{wire}");

        let mut cleared = M::default();
        cleared.clear(wire);
        assert_eq!(*cleared.field(wire), Some(None), "{wire}");
        assert!(cleared.is_cleared(wire), "{wire}");
        assert_eq!(
            serde_json::to_value(&cleared).unwrap(),
            json!({ wire: "" }),
            "{wire}"
        );

        // Setting after clearing (and vice versa) is last-write-wins.
        *cleared.field(wire) = Some(Some(date()));
        assert!(!cleared.is_cleared(wire), "{wire}");
        set.clear(wire);
        assert_eq!(
            serde_json::to_value(&set).unwrap(),
            json!({ wire: "" }),
            "{wire}"
        );
    }
}

fn deserializes_each_state<M: ClearableModel>() {
    for wire in FIELDS {
        let mut cleared: M = serde_json::from_value(json!({ wire: "" })).unwrap();
        assert!(cleared.is_cleared(wire), "{wire}");
        assert_eq!(*cleared.field(wire), Some(None), "{wire}");

        let mut set: M =
            serde_json::from_value(json!({ wire: "2026-12-01T09:00:00+02:00" })).unwrap();
        assert_eq!(*set.field(wire), Some(Some(date())), "{wire}");

        // `null` preserves the value on the server, exactly like omitting the property.
        let mut null: M = serde_json::from_value(json!({ wire: null })).unwrap();
        assert_eq!(*null.field(wire), None, "{wire}");
        let mut absent: M = serde_json::from_value(json!({})).unwrap();
        assert_eq!(*absent.field(wire), None, "{wire}");

        assert!(serde_json::from_value::<M>(json!({ wire: "not a date" })).is_err());

        // A clear survives a round trip.
        let back: M = serde_json::from_value(serde_json::to_value(&cleared).unwrap()).unwrap();
        assert!(back.is_cleared(wire), "{wire}");
    }
}

#[test]
fn job_result_corrections_serialize_set_clear_and_preserve() {
    serializes_each_state::<JobResultCorrections>();
}

#[test]
fn job_result_corrections_deserialize_an_empty_string_as_a_clear() {
    deserializes_each_state::<JobResultCorrections>();
}

#[test]
fn changeset_serializes_set_clear_and_preserve() {
    serializes_each_state::<Changeset>();
}

#[test]
fn changeset_deserializes_an_empty_string_as_a_clear() {
    deserializes_each_state::<Changeset>();
}

#[test]
fn user_task_update_request_sends_a_cleared_date_as_an_empty_string() {
    let mut changeset = Changeset::new();
    changeset.clear_due_date();
    changeset.follow_up_date = Some(Some(date()));
    changeset.priority = Some(Some(80));
    let request = UserTaskUpdateRequest {
        changeset: Some(Some(changeset)),
        ..UserTaskUpdateRequest::new()
    };
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        json!({
            "changeset": {
                "dueDate": "",
                "followUpDate": "2026-12-01T09:00:00+02:00",
                "priority": 80
            }
        })
    );
}

#[test]
fn a_hand_built_job_result_carries_its_clear() {
    let mut corrections = JobResultCorrections::new();
    corrections.clear_follow_up_date();
    let model = JobResult::UserTask(Box::new(JobResultUserTask {
        corrections: Some(Some(Box::new(corrections))),
        ..JobResultUserTask::new()
    }));
    let result = CompletionResult::from(model);
    assert!(result.clears_follow_up_date());
    assert!(!result.clears_due_date());
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        json!({ "type": "userTask", "corrections": { "followUpDate": "" } })
    );
}

#[test]
fn builder_clears_reach_the_generated_model() {
    let corrections = UserTaskCorrections::new().clear_due_date().into_inner();
    assert!(corrections.is_due_date_cleared());
    assert!(!corrections.is_follow_up_date_cleared());

    let result = CompletionResult::from(
        UserTaskResult::new().corrections(UserTaskCorrections::new().clear_due_date()),
    );
    assert!(result.clears_due_date());
    let JobResult::UserTask(user_task) = result.model().unwrap() else {
        panic!("expected a user-task result");
    };
    let corrections = user_task.corrections.clone().flatten().unwrap();
    assert!(corrections.is_due_date_cleared());
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        json!({ "type": "userTask", "corrections": { "dueDate": "" } })
    );
}
