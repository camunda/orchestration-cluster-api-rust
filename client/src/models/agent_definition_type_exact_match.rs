/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// AgentDefinitionTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentDefinitionTypeExactMatch(pub models::AgentDefinitionTypeEnum);

impl AgentDefinitionTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::AgentDefinitionTypeEnum) -> AgentDefinitionTypeExactMatch {
        AgentDefinitionTypeExactMatch(value)
    }
}
