/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// OperationTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OperationTypeExactMatch(pub models::AuditLogOperationTypeEnum);

impl OperationTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AuditLogOperationTypeEnum) -> OperationTypeExactMatch {
        OperationTypeExactMatch(value)
    }
}
