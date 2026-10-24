/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// UserTaskStateExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserTaskStateExactMatch(pub models::UserTaskStateEnum);

impl UserTaskStateExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::UserTaskStateEnum) -> UserTaskStateExactMatch {
        UserTaskStateExactMatch(value)
    }
}
