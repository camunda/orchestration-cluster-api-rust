/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// JobKindExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobKindExactMatch(pub models::JobKindEnum);

impl JobKindExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::JobKindEnum) -> JobKindExactMatch {
        JobKindExactMatch(value)
    }
}
