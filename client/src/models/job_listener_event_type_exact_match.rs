/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// JobListenerEventTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobListenerEventTypeExactMatch(pub models::JobListenerEventTypeEnum);

impl JobListenerEventTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::JobListenerEventTypeEnum) -> JobListenerEventTypeExactMatch {
        JobListenerEventTypeExactMatch(value)
    }
}
