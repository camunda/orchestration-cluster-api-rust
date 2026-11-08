//! Builders for the optional `result` a job worker sends when it completes a job.
//!
//! A completion can carry a [`models::JobResult`], a union discriminated on `type`:
//!
//! * [`UserTaskResult`] (`type: "userTask"`) -- a [user task listener] correcting task
//!   properties ([`UserTaskCorrections`]) or denying the work.
//! * [`AdHocSubProcessResult`] (`type: "adHocSubProcess"`) -- an ad-hoc sub-process job
//!   choosing which elements to activate and whether the sub-process is done.
//!
//! Both convert into [`models::JobResult`], so they are passed straight to
//! [`JobAction::complete_with_result`](super::job_worker::JobAction::complete_with_result).
//! The `type` discriminator comes from the enum variant, so it is always correct on the
//! wire. A hand-built [`models::JobResult`] is accepted too.
//!
//! [user task listener]: https://docs.camunda.io/docs/components/concepts/user-task-listeners/

use std::collections::HashMap;

use camunda_orchestration_api_client::models;
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

    /// The underlying generated model.
    pub fn into_inner(self) -> models::JobResultUserTask {
        self.inner
    }
}

impl From<UserTaskResult> for models::JobResult {
    fn from(result: UserTaskResult) -> Self {
        models::JobResult::UserTask(Box::new(result.inner))
    }
}

/// Task properties a user task listener corrects. An attribute left unset preserves the
/// persisted value.
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

    /// Correct the due date.
    pub fn due_date(mut self, due_date: impl Into<DateTime<FixedOffset>>) -> Self {
        self.inner.due_date = Some(Some(due_date.into()));
        self
    }

    /// Correct the follow-up date.
    pub fn follow_up_date(mut self, follow_up_date: impl Into<DateTime<FixedOffset>>) -> Self {
        self.inner.follow_up_date = Some(Some(follow_up_date.into()));
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

    /// The underlying generated model.
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
    /// serialize to a JSON object; anything else is sent without variables.
    pub fn activate_element_with(
        self,
        element_id: impl Into<String>,
        variables: impl Serialize,
    ) -> Self {
        let variables = match serde_json::to_value(variables) {
            Ok(Value::Object(map)) => Some(map.into_iter().collect()),
            _ => None,
        };
        self.push_element(element_id.into(), variables)
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
    pub fn into_inner(self) -> models::JobResultAdHocSubProcess {
        self.inner
    }
}

impl From<AdHocSubProcessResult> for models::JobResult {
    fn from(result: AdHocSubProcessResult) -> Self {
        models::JobResult::AdHocSubProcess(Box::new(result.inner))
    }
}
