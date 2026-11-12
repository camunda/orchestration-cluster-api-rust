/*
 * Orchestration Cluster API
 *
 * @generated clearable fields (scripts/postprocess.py hook 19) — DO NOT EDIT.
 */

//! Serde support for fields the API clears with an empty string.

/// Serde for a clearable date-time field typed
/// `Option<Option<chrono::DateTime<chrono::FixedOffset>>>`:
///
/// | Value | JSON | Meaning |
/// | --- | --- | --- |
/// | `None` | omitted | preserve the persisted value |
/// | `Some(Some(date))` | RFC 3339 string | set the value |
/// | `Some(None)` | `""` | clear the persisted value |
///
/// Deserializing maps `""` to `Some(None)` and `null` (which the API also treats as
/// "preserve") to `None`.
pub mod date_time {
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        value: &Option<Option<DateTime<FixedOffset>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(Some(date)) => date.serialize(serializer),
            Some(None) => serializer.serialize_str(""),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Option<DateTime<FixedOffset>>>, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(None),
            Some(s) if s.is_empty() => Ok(Some(None)),
            Some(s) => s
                .parse::<DateTime<FixedOffset>>()
                .map(|date| Some(Some(date)))
                .map_err(serde::de::Error::custom),
        }
    }
}
