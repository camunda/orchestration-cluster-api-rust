/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// VariableHistoryItemOperationTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariableHistoryItemOperationTypeExactMatch(
    pub models::VariableHistoryItemOperationTypeEnum,
);

impl VariableHistoryItemOperationTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(
        value: models::VariableHistoryItemOperationTypeEnum,
    ) -> VariableHistoryItemOperationTypeExactMatch {
        VariableHistoryItemOperationTypeExactMatch(value)
    }
}
