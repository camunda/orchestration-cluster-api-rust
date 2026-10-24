/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// EntityTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityTypeExactMatch(pub models::AuditLogEntityTypeEnum);

impl EntityTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AuditLogEntityTypeEnum) -> EntityTypeExactMatch {
        EntityTypeExactMatch(value)
    }
}
