/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// MessageSubscriptionTypeExactMatch : Matches the value exactly.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageSubscriptionTypeExactMatch(pub models::MessageSubscriptionTypeEnum);

impl MessageSubscriptionTypeExactMatch {
    /// Matches the value exactly.
    pub fn new(value: models::MessageSubscriptionTypeEnum) -> MessageSubscriptionTypeExactMatch {
        MessageSubscriptionTypeExactMatch(value)
    }
}
