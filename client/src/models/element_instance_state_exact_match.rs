/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ElementInstanceStateExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ElementInstanceStateExactMatch(pub models::ElementInstanceStateEnum);

impl ElementInstanceStateExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::ElementInstanceStateEnum) -> ElementInstanceStateExactMatch {
        ElementInstanceStateExactMatch(value)
    }
}
