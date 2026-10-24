/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// IncidentStateExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IncidentStateExactMatch(pub models::IncidentStateEnum);

impl IncidentStateExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::IncidentStateEnum) -> IncidentStateExactMatch {
        IncidentStateExactMatch(value)
    }
}
