//! Builders for the optional `result` a job worker sends when it completes a job.
//!
//! A completion can carry a [`models::JobResult`], a union discriminated on `type`:
//!
//! * [`UserTaskResult`] (`type: "userTask"`) -- a [user task listener] correcting task
//!   properties ([`UserTaskCorrections`]) or denying the work.
//! * [`AdHocSubProcessResult`] (`type: "adHocSubProcess"`) -- an ad-hoc sub-process job
//!   choosing which elements to activate and whether the sub-process is done.
//!
//! Both convert into a [`CompletionResult`], which is what
//! [`JobAction::complete_with_result`](super::job_worker::JobAction::complete_with_result)
//! takes. A hand-built [`models::JobResult`] converts too. The `type` discriminator comes
//! from the generated enum variant, so it is always correct on the wire.
//!
//! The API clears a corrected `dueDate` / `followUpDate` with an empty string, while
//! `null` or omitting it preserves the value. The generated
//! [`models::JobResultCorrections`] represents a clear as `Some(None)` and sends it as `""`
//! ([`UserTaskCorrections::clear_due_date`] sets it).
//!
//! [user task listener]: https://docs.camunda.io/docs/components/concepts/user-task-listeners/

use std::collections::HashMap;

use camunda_orchestration_api_client::models;

use super::errors::CamundaError;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use serde_json::Value;

/// The result of a job raised by a user task listener (`type: "userTask"`).
///
/// ```
/// use camunda_orchestration_sdk::{JobAction, UserTaskCorrections, UserTaskResult};
///
/// let correct = JobAction::complete_with_result(
///     UserTaskResult::new().corrections(UserTaskCorrections::new().assignee("alice")),
/// );
/// let deny = JobAction::complete_with_result(UserTaskResult::deny("Missing documents"));
/// # let _ = (correct, deny);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserTaskResult {
    inner: models::JobResultUserTask,
}

impl UserTaskResult {
    /// An empty user-task result: the work is approved and nothing is corrected.
    pub fn new() -> Self {
        Self::default()
    }

    /// Deny the work (for example, reject a task completion), giving a reason.
    pub fn deny(reason: impl Into<String>) -> Self {
        Self::new().denied(true).denied_reason(reason)
    }

    /// Set whether the worker denies the work. Defaults to `false` on the engine.
    pub fn denied(mut self, denied: bool) -> Self {
        self.inner.denied = Some(Some(denied));
        self
    }

    /// The reason for denying the work.
    pub fn denied_reason(mut self, reason: impl Into<String>) -> Self {
        self.inner.denied_reason = Some(Some(reason.into()));
        self
    }

    /// Correct task properties as part of the completion.
    pub fn corrections(mut self, corrections: UserTaskCorrections) -> Self {
        self.inner.corrections = Some(Some(Box::new(corrections.inner)));
        self
    }

    /// The underlying generated model, including any cleared due or follow-up date.
    pub fn into_inner(self) -> models::JobResultUserTask {
        self.inner
    }
}

/// Task properties a user task listener corrects. An attribute left unset preserves the
/// persisted value.
///
/// For the due and follow-up dates, setting and clearing the same date overrides each
/// other: the last call wins.
///
/// ```
/// use camunda_orchestration_sdk::UserTaskCorrections;
///
/// let corrections = UserTaskCorrections::new()
///     .clear_due_date() // sent as "dueDate": ""
///     .priority(80);
/// # let _ = corrections;
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserTaskCorrections {
    inner: models::JobResultCorrections,
}

impl UserTaskCorrections {
    /// No corrections.
    pub fn new() -> Self {
        Self::default()
    }

    /// Correct the assignee. An empty string clears it.
    pub fn assignee(mut self, assignee: impl Into<String>) -> Self {
        self.inner.assignee = Some(Some(assignee.into()));
        self
    }

    /// Correct the due date. Overrides an earlier [`clear_due_date`](Self::clear_due_date).
    pub fn due_date(mut self, due_date: impl Into<DateTime<FixedOffset>>) -> Self {
        self.inner.due_date = Some(Some(due_date.into()));
        self
    }

    /// Clear the due date (sent as `"dueDate": ""`). Overrides an earlier
    /// [`due_date`](Self::due_date).
    pub fn clear_due_date(mut self) -> Self {
        self.inner.clear_due_date();
        self
    }

    /// Correct the follow-up date. Overrides an earlier
    /// [`clear_follow_up_date`](Self::clear_follow_up_date).
    pub fn follow_up_date(mut self, follow_up_date: impl Into<DateTime<FixedOffset>>) -> Self {
        self.inner.follow_up_date = Some(Some(follow_up_date.into()));
        self
    }

    /// Clear the follow-up date (sent as `"followUpDate": ""`). Overrides an earlier
    /// [`follow_up_date`](Self::follow_up_date).
    pub fn clear_follow_up_date(mut self) -> Self {
        self.inner.clear_follow_up_date();
        self
    }

    /// Correct the candidate users. An empty list clears them.
    pub fn candidate_users<I, S>(mut self, users: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner.candidate_users = Some(Some(users.into_iter().map(Into::into).collect()));
        self
    }

    /// Correct the candidate groups. An empty list clears them.
    pub fn candidate_groups<I, S>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner.candidate_groups = Some(Some(groups.into_iter().map(Into::into).collect()));
        self
    }

    /// Correct the priority (0-100).
    pub fn priority(mut self, priority: i32) -> Self {
        self.inner.priority = Some(Some(priority));
        self
    }

    /// The underlying generated model. A [`clear_due_date`](Self::clear_due_date) /
    /// [`clear_follow_up_date`](Self::clear_follow_up_date) is carried as `Some(None)`.
    pub fn into_inner(self) -> models::JobResultCorrections {
        self.inner
    }
}

/// The result of a job raised by an ad-hoc sub-process (`type: "adHocSubProcess"`).
///
/// ```
/// use camunda_orchestration_sdk::{AdHocSubProcessResult, JobAction};
///
/// let action = JobAction::complete_with_result(
///     AdHocSubProcessResult::new()
///         .activate_element_with("review", serde_json::json!({ "reviewer": "alice" }))
///         .completion_condition_fulfilled(false),
/// );
/// # let _ = action;
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AdHocSubProcessResult {
    inner: models::JobResultAdHocSubProcess,
    /// The first element-variables serialization failure, if any.
    error: Option<String>,
}

impl AdHocSubProcessResult {
    /// An empty ad-hoc sub-process result: nothing is activated.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate the element with this id.
    pub fn activate_element(self, element_id: impl Into<String>) -> Self {
        self.push_element(element_id.into(), None)
    }

    /// Activate the element with this id, with variables scoped to it. `variables` must
    /// serialize to a JSON object; any other value is sent without variables.
    ///
    /// If `variables` fails to serialize, completing the job with this result fails the
    /// job instead (see [`JobAction::complete_with_result`]), rather than activating the
    /// element without the variables it was meant to have.
    ///
    /// [`JobAction::complete_with_result`]: super::job_worker::JobAction::complete_with_result
    pub fn activate_element_with(
        mut self,
        element_id: impl Into<String>,
        variables: impl Serialize,
    ) -> Self {
        let element_id = element_id.into();
        let variables = match serde_json::to_value(variables) {
            Ok(Value::Object(map)) => Some(map.into_iter().collect()),
            Ok(_) => None,
            Err(e) => {
                self.error.get_or_insert_with(|| {
                    format!("failed to serialize variables for element '{element_id}': {e}")
                });
                None
            }
        };
        self.push_element(element_id, variables)
    }

    fn push_element(
        mut self,
        element_id: String,
        variables: Option<HashMap<String, Value>>,
    ) -> Self {
        self.inner
            .activate_elements
            .get_or_insert_with(Vec::new)
            .push(models::JobResultActivateElement {
                element_id: Some(models::ElementId::assume_exists(element_id)),
                variables: variables.map(Some),
            });
        self
    }

    /// Set whether the completion condition of the ad-hoc sub-process is fulfilled.
    pub fn completion_condition_fulfilled(mut self, fulfilled: bool) -> Self {
        self.inner.is_completion_condition_fulfilled = Some(fulfilled);
        self
    }

    /// Set whether the remaining instances of the ad-hoc sub-process are canceled.
    pub fn cancel_remaining_instances(mut self, cancel: bool) -> Self {
        self.inner.is_cancel_remaining_instances = Some(cancel);
        self
    }

    /// The underlying generated model.
    ///
    /// # Errors
    ///
    /// [`CamundaError::Validation`] if element variables passed to
    /// [`activate_element_with`](Self::activate_element_with) failed to serialize: the
    /// model would activate that element without them.
    pub fn into_inner(self) -> Result<models::JobResultAdHocSubProcess, CamundaError> {
        match self.error {
            Some(error) => Err(CamundaError::Validation(error)),
            None => Ok(self.inner),
        }
    }
}

/// The job result a completion carries: the generated [`models::JobResult`], plus any
/// failure a builder hit while assembling it.
///
/// Build one from a [`UserTaskResult`], an [`AdHocSubProcessResult`], or a
/// [`models::JobResult`]. It serializes to the wire form of the completion `result`.
///
/// If a builder failed to serialize part of the result, every way of turning it into a
/// payload fails rather than dropping that part: serializing returns a serde error,
/// [`model`](Self::model) returns the error, and
/// [`JobAction::complete_with_result`](super::job_worker::JobAction::complete_with_result)
/// fails the job.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionResult {
    model: models::JobResult,
    /// Why the result cannot be sent (a serialization failure in a builder).
    error: Option<String>,
}

impl CompletionResult {
    /// The generated model, including any cleared due or follow-up date.
    ///
    /// # Errors
    ///
    /// [`CamundaError::Validation`] if a builder failed to serialize part of the result
    /// (see [`error`](Self::error)): the model is missing that part.
    pub fn model(&self) -> Result<&models::JobResult, CamundaError> {
        match &self.error {
            Some(error) => Err(CamundaError::Validation(error.clone())),
            None => Ok(&self.model),
        }
    }

    /// Why this result cannot be sent, if a builder failed to serialize part of it.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Whether the due date is cleared.
    pub fn clears_due_date(&self) -> bool {
        self.corrections()
            .is_some_and(models::JobResultCorrections::is_due_date_cleared)
    }

    /// Whether the follow-up date is cleared.
    pub fn clears_follow_up_date(&self) -> bool {
        self.corrections()
            .is_some_and(models::JobResultCorrections::is_follow_up_date_cleared)
    }

    fn corrections(&self) -> Option<&models::JobResultCorrections> {
        match &self.model {
            models::JobResult::UserTask(result) => result.corrections.as_ref()?.as_deref(),
            _ => None,
        }
    }
}

impl Serialize for CompletionResult {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.error {
            Some(error) => Err(serde::ser::Error::custom(error)),
            None => self.model.serialize(serializer),
        }
    }
}

impl From<models::JobResult> for CompletionResult {
    fn from(model: models::JobResult) -> Self {
        Self { model, error: None }
    }
}

impl From<UserTaskResult> for CompletionResult {
    fn from(result: UserTaskResult) -> Self {
        Self {
            model: models::JobResult::UserTask(Box::new(result.inner)),
            error: None,
        }
    }
}

impl From<AdHocSubProcessResult> for CompletionResult {
    fn from(result: AdHocSubProcessResult) -> Self {
        Self {
            model: models::JobResult::AdHocSubProcess(Box::new(result.inner)),
            error: result.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A value whose `Serialize` impl always fails.
    struct Unserializable;

    impl Serialize for Unserializable {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("boom"))
        }
    }

    fn broken() -> AdHocSubProcessResult {
        AdHocSubProcessResult::new()
            .activate_element("first")
            .activate_element_with("review", Unserializable)
    }

    #[test]
    fn a_builder_error_blocks_every_payload_path() {
        let result = CompletionResult::from(broken());
        let expected = "failed to serialize variables for element 'review': boom";
        assert_eq!(result.error(), Some(expected));

        let serialized = serde_json::to_value(&result).unwrap_err().to_string();
        assert!(serialized.contains(expected), "{serialized}");
        assert!(
            matches!(result.model(), Err(CamundaError::Validation(m)) if m == expected),
            "model() must not expose the incomplete model"
        );
        assert!(matches!(
            broken().into_inner(),
            Err(CamundaError::Validation(m)) if m == expected
        ));
    }

    #[test]
    fn a_sound_result_serializes_and_exposes_its_model() {
        let result = CompletionResult::from(
            AdHocSubProcessResult::new()
                .activate_element_with("review", serde_json::json!({"a": 1})),
        );
        assert_eq!(result.error(), None);
        assert!(result.model().is_ok());
        assert_eq!(
            serde_json::to_value(&result).unwrap(),
            serde_json::json!({
                "type": "adHocSubProcess",
                "activateElements": [{ "elementId": "review", "variables": { "a": 1 } }]
            })
        );
        assert!(AdHocSubProcessResult::new().into_inner().is_ok());
    }
}
