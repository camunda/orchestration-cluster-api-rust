/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// VariableHistoryItemActorTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariableHistoryItemActorTypeExactMatch(pub models::VariableHistoryItemActorTypeEnum);

impl VariableHistoryItemActorTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(
        value: models::VariableHistoryItemActorTypeEnum,
    ) -> VariableHistoryItemActorTypeExactMatch {
        VariableHistoryItemActorTypeExactMatch(value)
    }
}
