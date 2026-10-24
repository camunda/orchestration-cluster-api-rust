/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ClusterVariableKindExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClusterVariableKindExactMatch(pub models::ClusterVariableKindEnum);

impl ClusterVariableKindExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::ClusterVariableKindEnum) -> ClusterVariableKindExactMatch {
        ClusterVariableKindExactMatch(value)
    }
}
