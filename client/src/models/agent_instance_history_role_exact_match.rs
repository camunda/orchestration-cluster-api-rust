/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// AgentInstanceHistoryRoleExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentInstanceHistoryRoleExactMatch(pub models::AgentInstanceHistoryRoleEnum);

impl AgentInstanceHistoryRoleExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AgentInstanceHistoryRoleEnum) -> AgentInstanceHistoryRoleExactMatch {
        AgentInstanceHistoryRoleExactMatch(value)
    }
}
