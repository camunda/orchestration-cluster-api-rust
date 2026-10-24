/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// BatchOperationTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BatchOperationTypeExactMatch(pub models::BatchOperationTypeEnum);

impl BatchOperationTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::BatchOperationTypeEnum) -> BatchOperationTypeExactMatch {
        BatchOperationTypeExactMatch(value)
    }
}
