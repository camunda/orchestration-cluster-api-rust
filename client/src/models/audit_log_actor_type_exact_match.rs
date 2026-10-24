/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// AuditLogActorTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditLogActorTypeExactMatch(pub models::AuditLogActorTypeEnum);

impl AuditLogActorTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AuditLogActorTypeEnum) -> AuditLogActorTypeExactMatch {
        AuditLogActorTypeExactMatch(value)
    }
}
