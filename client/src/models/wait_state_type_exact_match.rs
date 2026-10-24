/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// WaitStateTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WaitStateTypeExactMatch(pub models::WaitStateTypeEnum);

impl WaitStateTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::WaitStateTypeEnum) -> WaitStateTypeExactMatch {
        WaitStateTypeExactMatch(value)
    }
}
