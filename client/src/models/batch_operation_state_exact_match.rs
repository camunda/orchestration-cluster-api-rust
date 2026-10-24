/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// BatchOperationStateExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BatchOperationStateExactMatch(pub models::BatchOperationStateEnum);

impl BatchOperationStateExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::BatchOperationStateEnum) -> BatchOperationStateExactMatch {
        BatchOperationStateExactMatch(value)
    }
}
