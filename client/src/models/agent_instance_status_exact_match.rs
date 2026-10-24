/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// AgentInstanceStatusExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentInstanceStatusExactMatch(pub models::AgentInstanceStatusEnum);

impl AgentInstanceStatusExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AgentInstanceStatusEnum) -> AgentInstanceStatusExactMatch {
        AgentInstanceStatusExactMatch(value)
    }
}
