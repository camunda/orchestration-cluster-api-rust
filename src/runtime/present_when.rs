//! Runtime enforcement of the dependent-presence couplings the specification declares
//! with `x-present-when` (see `present_when_generated.rs`).
//!
//! Rust has no dependent typing over a runtime-configured flag, so the coupling — a field
//! present exactly when a request set a flag — is enforced here at the activation boundary
//! rather than expressed in the type system. The generated table is the ground truth; the
//! tests in this module assert the runtime still matches it, so an upstream change to the
//! markers fails the build rather than drifting silently.

use super::errors::CamundaError;
use super::present_when_generated::{PresentWhenCoupling, PRESENT_WHEN_COUPLINGS};

/// The one coupling this runtime enforces: `ActivatedJobResult.jobLeaseToken` is present
/// only when an activation set the lease flag.
pub(crate) const LEASE_COUPLING_KEY: &str = "ActivatedJobResult.jobLeaseToken";

/// `Schema.field` key for a coupling.
fn coupling_key(c: &PresentWhenCoupling) -> String {
    format!("{}.{}", c.response_schema, c.response_field)
}

/// The request flag that governs the lease coupling, read from the generated table so a
/// rename upstream is reflected here rather than hardcoded. Falls back to the literal only
/// if the table has drifted, which `generated_table_matches_the_spec` fails on separately.
fn lease_request_flag() -> &'static str {
    PRESENT_WHEN_COUPLINGS
        .iter()
        .find(|c| coupling_key(c) == LEASE_COUPLING_KEY)
        .map(|c| c.request_flag)
        .unwrap_or("withLease")
}

/// Enforce the lease coupling for a single activated job.
///
/// `requested` reports whether the activation set the lease flag; `token` is the lease
/// token the server returned for the job. A lease that was asked for but not returned is
/// rejected, because every fenced command would otherwise go out unfenced.
pub(crate) fn require_lease_presence(
    requested: bool,
    job_key: &str,
    token: Option<&str>,
) -> Result<(), CamundaError> {
    if !requested || token.is_some_and(|t| !t.is_empty()) {
        return Ok(());
    }
    Err(CamundaError::LeaseNotHonored {
        job_key: job_key.to_string(),
        request_flag: lease_request_flag(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Couplings the runtime actively enforces, by `Schema.field` key. A coupling in
    /// [`PRESENT_WHEN_COUPLINGS`] that is absent here is one the specification declares and
    /// the worker silently ignores; [`every_declared_coupling_is_enforced`] makes that
    /// visible. Verification metadata only — the runtime path is hardcoded to the lease.
    const ENFORCED_COUPLINGS: &[&str] = &[LEASE_COUPLING_KEY];

    /// A coupling re-derived from the spec, independent of the generated table.
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SpecCoupling {
        schema: String,
        field: String,
        flag: String,
    }

    /// Re-derive the couplings straight from the bundled spec, so the guard does not lean
    /// on the same code path that produced the table it checks.
    fn couplings_declared_by_spec() -> BTreeSet<SpecCoupling> {
        let raw = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/external-spec/bundled/rest-api.bundle.json"
        ))
        .expect("bundled spec must be readable");
        let spec: serde_json::Value = serde_json::from_str(&raw).expect("bundled spec must parse");

        let schemas = spec["components"]["schemas"]
            .as_object()
            .expect("spec has components.schemas");
        let mut out = BTreeSet::new();
        for (schema_name, schema) in schemas {
            let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
                continue;
            };
            for (field_name, field) in props {
                let Some(marker) = field.get("x-present-when") else {
                    continue;
                };
                let flag = marker
                    .get("request")
                    .and_then(|r| r.as_str())
                    .expect("x-present-when marker has a string `request` flag");
                out.insert(SpecCoupling {
                    schema: schema_name.clone(),
                    field: field_name.clone(),
                    flag: flag.to_string(),
                });
            }
        }
        out
    }

    /// The derivation guard, scoped to the class of defect — a coupling the spec declares
    /// that the generated table does not carry — rather than to the lease instance. A
    /// second `x-present-when` field added upstream cannot slip past the generator unseen.
    #[test]
    fn generated_table_matches_the_spec() {
        let declared = couplings_declared_by_spec();
        assert!(
            !declared.is_empty(),
            "bundled spec declares no x-present-when markers: the bundler is stripping the \
             vendor key or upstream dropped it, and this guard must not pass vacuously"
        );
        let generated: BTreeSet<SpecCoupling> = PRESENT_WHEN_COUPLINGS
            .iter()
            .map(|c| SpecCoupling {
                schema: c.response_schema.to_string(),
                field: c.response_field.to_string(),
                flag: c.request_flag.to_string(),
            })
            .collect();
        assert_eq!(
            generated, declared,
            "the generated present-when table has drifted from the spec markers"
        );
    }

    /// Deriving a coupling is worthless if no runtime path acts on it, so a coupling the
    /// spec declares and the runtime does not enforce fails here rather than silently
    /// handing callers an unfenced job.
    #[test]
    fn every_declared_coupling_is_enforced() {
        for c in PRESENT_WHEN_COUPLINGS {
            let key = coupling_key(c);
            assert!(
                ENFORCED_COUPLINGS.contains(&key.as_str()),
                "spec declares dependent presence for {key} (when `{}` is set) but no runtime \
                 path enforces it: a caller opting in would receive an unfenced job",
                c.request_flag
            );
        }
    }

    /// The mirror-image drift: an enforcement entry naming a coupling the spec no longer
    /// declares would leave dead runtime code asserting a contract that has moved.
    #[test]
    fn every_enforced_coupling_is_declared() {
        let declared: BTreeSet<String> = PRESENT_WHEN_COUPLINGS.iter().map(coupling_key).collect();
        for key in ENFORCED_COUPLINGS {
            assert!(
                declared.contains(*key),
                "runtime enforces {key} but the spec no longer declares it"
            );
        }
    }

    #[test]
    fn require_lease_presence_covers_every_case() {
        // lease requested and honored
        assert!(require_lease_presence(true, "1", Some("tok")).is_ok());
        // lease not requested, none returned
        assert!(require_lease_presence(false, "1", None).is_ok());
        // lease not requested, one returned anyway (harmless)
        assert!(require_lease_presence(false, "1", Some("tok")).is_ok());
        // lease requested but empty token is as good as none
        assert!(matches!(
            require_lease_presence(true, "1", Some("")),
            Err(CamundaError::LeaseNotHonored { .. })
        ));
        // lease requested but not honored
        match require_lease_presence(true, "job-9", None) {
            Err(CamundaError::LeaseNotHonored {
                job_key,
                request_flag,
            }) => {
                assert_eq!(job_key, "job-9");
                assert_eq!(request_flag, "withLease");
            }
            other => panic!("expected LeaseNotHonored, got {other:?}"),
        }
    }

    #[test]
    fn lease_flag_is_read_from_the_table() {
        assert_eq!(lease_request_flag(), "withLease");
    }
}
