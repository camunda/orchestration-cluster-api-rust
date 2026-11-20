//! Spec-derived guard: every operation whose request body (or a union variant of it)
//! carries an *optional* top-level `tenantId` must apply the effective default tenant.
//!
//! The set of operations is computed from the bundled spec, not from a list, so a new
//! tenant-scoped operation fails this test until hook 07 (or a curated wrapper) injects it.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

fn resolve<'a>(mut schema: &'a Value, schemas: &'a Value) -> &'a Value {
    let mut hops = 0;
    while let Some(r) = schema.get("$ref").and_then(Value::as_str) {
        schema = &schemas[r.rsplit('/').next().unwrap()];
        hops += 1;
        assert!(hops < 64, "ref cycle at {r}");
    }
    schema
}

fn has_optional_tenant(schema: &Value, schemas: &Value) -> bool {
    fn collect(s: &Value, schemas: &Value, props: &mut bool, req: &mut bool) {
        let s = resolve(s, schemas);
        if s.pointer("/properties/tenantId").is_some() {
            *props = true;
        }
        if let Some(r) = s.get("required").and_then(Value::as_array) {
            if r.iter().any(|v| v == "tenantId") {
                *req = true;
            }
        }
        for p in s
            .get("allOf")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            collect(p, schemas, props, req);
        }
    }
    let (mut props, mut req) = (false, false);
    collect(schema, schemas, &mut props, &mut req);
    props && !req
}

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

fn tenant_scoped_operations(spec: &Value) -> BTreeSet<String> {
    let schemas = &spec["components"]["schemas"];
    let mut out = BTreeSet::new();
    for item in spec["paths"].as_object().unwrap().values() {
        for op in item.as_object().unwrap().values() {
            let Some(op_id) = op.get("operationId").and_then(Value::as_str) else {
                continue;
            };
            let Some(content) = op
                .pointer("/requestBody/content")
                .and_then(Value::as_object)
            else {
                continue;
            };
            for media in content.values() {
                let body = &media["schema"];
                let resolved = resolve(body, schemas);
                let union = resolved
                    .get("oneOf")
                    .or_else(|| resolved.get("anyOf"))
                    .and_then(Value::as_array);
                let scoped = match union {
                    Some(vs) => vs.iter().any(|v| has_optional_tenant(v, schemas)),
                    None => has_optional_tenant(body, schemas),
                };
                if scoped {
                    out.insert(snake_case(op_id));
                }
            }
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
    let spec: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("external-spec/bundled/rest-api.bundle.json")).unwrap(),
    )
    .unwrap();
    let facade = std::fs::read_to_string(root.join("src/runtime/facade_generated.rs")).unwrap();
    let curated = std::fs::read_to_string(root.join("src/runtime/client.rs")).unwrap();

    let ops = tenant_scoped_operations(&spec);
    for expected in ["suspend_process_definitions", "resume_process_definitions"] {
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
        if !(body.contains("default_tenant") || body.contains("_tenant(")) {
            missing.push(op.clone());
        }
    }
    assert!(
        missing.is_empty(),
        "default tenant not injected for: {missing:?}"
    );
}
