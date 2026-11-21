//! Spec-derived guard: every operation whose request body (or a union variant of it)
//! carries an *optional* top-level `tenantId` (or `tenantIds`) must apply the effective
//! default tenant.
//!
//! The set of operations is read from the bundler's `spec-metadata.json`
//! (`optionalTenantIdInBodyVariants` / `optionalTenantIdsInBody`), not from a list, so a new
//! tenant-scoped operation fails this test until hook 07 (or a curated wrapper) injects it.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

fn snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn tenant_scoped_operations(metadata: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for op in metadata["operations"].as_array().unwrap() {
        let op_id = op["operationId"].as_str().unwrap();
        let variants = op["optionalTenantIdInBodyVariants"]
            .as_array()
            .unwrap_or_else(|| panic!("{op_id}: no optionalTenantIdInBodyVariants"));
        let tenant_ids = op["optionalTenantIdsInBody"]
            .as_bool()
            .unwrap_or_else(|| panic!("{op_id}: no optionalTenantIdsInBody"));
        if !variants.is_empty() || tenant_ids {
            out.insert(snake_case(op_id));
        }
    }
    out
}

/// The body of `pub async fn <name>(` up to the next public item.
fn method_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let start = src.find(&format!("pub async fn {name}("))?;
    let rest = &src[start..];
    let end = rest[1..].find("\n    pub ").map_or(rest.len(), |i| i + 1);
    Some(&rest[..end])
}

#[test]
fn every_optional_body_tenant_operation_injects_the_default_tenant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let metadata: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("external-spec/bundled/spec-metadata.json")).unwrap(),
    )
    .unwrap();
    let facade = std::fs::read_to_string(root.join("src/runtime/facade_generated.rs")).unwrap();
    let curated = std::fs::read_to_string(root.join("src/runtime/client.rs")).unwrap();

    let ops = tenant_scoped_operations(&metadata);
    for expected in [
        "suspend_process_definitions",
        "resume_process_definitions",
        "activate_jobs",
    ] {
        assert!(
            ops.contains(expected),
            "{expected} not detected as tenant-scoped"
        );
    }
    assert!(
        !ops.contains("create_tenant"),
        "required tenantId must not be injected"
    );

    let mut missing = Vec::new();
    for op in &ops {
        let body = method_body(&facade, op)
            .or_else(|| method_body(&curated, op))
            .unwrap_or_else(|| panic!("no client method for {op}"));
        if !(body.contains("default_tenant")
            || body.contains("_tenant(")
            || body.contains("_tenants("))
        {
            missing.push(op.clone());
        }
    }
    assert!(
        missing.is_empty(),
        "default tenant not injected for: {missing:?}"
    );
}
