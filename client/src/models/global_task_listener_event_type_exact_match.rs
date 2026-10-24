/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 15) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// GlobalTaskListenerEventTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GlobalTaskListenerEventTypeExactMatch(pub models::GlobalTaskListenerEventTypeEnum);

impl GlobalTaskListenerEventTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(
        value: models::GlobalTaskListenerEventTypeEnum,
    ) -> GlobalTaskListenerEventTypeExactMatch {
        GlobalTaskListenerEventTypeExactMatch(value)
    }
}
