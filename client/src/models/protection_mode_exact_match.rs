/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ProtectionModeExactMatch : Matches variables that have this protection mode.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtectionModeExactMatch(pub models::ProtectionModeEnum);

impl ProtectionModeExactMatch {
    /// Matches variables that have this protection mode.
    pub fn new(value: models::ProtectionModeEnum) -> ProtectionModeExactMatch {
        ProtectionModeExactMatch(value)
    }
}
