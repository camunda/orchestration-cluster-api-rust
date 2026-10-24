/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// AgentInstanceHistoryCommitStatusExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentInstanceHistoryCommitStatusExactMatch(
    pub models::AgentInstanceHistoryCommitStatusEnum,
);

impl AgentInstanceHistoryCommitStatusExactMatch {
    /// Matches the value exactly.
    pub fn new(
        value: models::AgentInstanceHistoryCommitStatusEnum,
    ) -> AgentInstanceHistoryCommitStatusExactMatch {
        AgentInstanceHistoryCommitStatusExactMatch(value)
    }
}
