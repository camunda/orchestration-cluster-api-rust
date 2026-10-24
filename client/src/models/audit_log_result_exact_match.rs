/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// AuditLogResultExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditLogResultExactMatch(pub models::AuditLogResultEnum);

impl AuditLogResultExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AuditLogResultEnum) -> AuditLogResultExactMatch {
        AuditLogResultExactMatch(value)
    }
}
