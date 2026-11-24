/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// VariableHistoryItemVariableOperationTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariableHistoryItemVariableOperationTypeExactMatch(
    pub models::VariableHistoryItemVariableOperationTypeEnum,
);

impl VariableHistoryItemVariableOperationTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(
        value: models::VariableHistoryItemVariableOperationTypeEnum,
    ) -> VariableHistoryItemVariableOperationTypeExactMatch {
        VariableHistoryItemVariableOperationTypeExactMatch(value)
    }
}
