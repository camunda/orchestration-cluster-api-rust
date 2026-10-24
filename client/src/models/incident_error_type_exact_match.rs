/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// IncidentErrorTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IncidentErrorTypeExactMatch(pub models::IncidentErrorTypeEnum);

impl IncidentErrorTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::IncidentErrorTypeEnum) -> IncidentErrorTypeExactMatch {
        IncidentErrorTypeExactMatch(value)
    }
}
