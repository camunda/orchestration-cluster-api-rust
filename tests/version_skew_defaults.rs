//! Responses from servers that predate a newly required field must still
//! deserialize. Hook 08 (version-skew-tolerance) adds `#[serde(default)]` to
//! these fields; this guards them against regeneration drift.

use camunda_orchestration_sdk::models::CamundaUserResult;

#[test]
fn deserializes_authenticated_user_without_is_anonymous() {
    // `isAnonymous` was added in 8.10.10; earlier servers omit it.
    let value = serde_json::json!({
        "username": "demo",
        "displayName": "Demo User",
        "email": "demo@example.com",
        "authorizedComponents": ["*"],
        "tenants": [],
        "groups": [],
        "roles": [],
        "salesPlanType": null,
        "c8Links": {},
        "canLogout": true
    });
    let user: CamundaUserResult = serde_json::from_value(value)
        .expect("a /authentication/me response without `isAnonymous` must deserialize");
    assert!(
        !user.is_anonymous,
        "absent isAnonymous must default to false"
    );
}
