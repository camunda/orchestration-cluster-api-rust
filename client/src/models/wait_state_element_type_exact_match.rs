/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// WaitStateElementTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WaitStateElementTypeExactMatch(pub models::WaitStateElementTypeEnum);

impl WaitStateElementTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::WaitStateElementTypeEnum) -> WaitStateElementTypeExactMatch {
        WaitStateElementTypeExactMatch(value)
    }
}
