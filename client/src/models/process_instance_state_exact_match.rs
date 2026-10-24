/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ProcessInstanceStateExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProcessInstanceStateExactMatch(pub models::ProcessInstanceStateEnum);

impl ProcessInstanceStateExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::ProcessInstanceStateEnum) -> ProcessInstanceStateExactMatch {
        ProcessInstanceStateExactMatch(value)
    }
}
