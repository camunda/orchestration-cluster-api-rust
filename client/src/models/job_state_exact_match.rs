/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// JobStateExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobStateExactMatch(pub models::JobStateEnum);

impl JobStateExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::JobStateEnum) -> JobStateExactMatch {
        JobStateExactMatch(value)
    }
}
