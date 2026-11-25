# ActivatedProbeResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**probe_key** | **models::ProbeKey** | The key to answer the probe with. | 
**probe_id** | **String** | The probe ID, to pick the handler when the worker polls several probe IDs. | 
**worker** | **String** | The name of the worker that collected the probe. | 
**deadline** | **i64** | The deadline of the probe, as a UNIX epoch timestamp in milliseconds. An answer after it gets 404.  | 
**tenant_id** | **String** | The ID of the tenant of the probe. | 
**physical_tenant_id** | **String** | The ID of the physical tenant that the activation request was routed to; the default physical tenant when the request did not specify one.  | 
**input** | **std::collections::HashMap<String, serde_json::Value>** | The evaluated input, with the values of its secrets injected. | 
**secret_pointers** | **Vec<String>** | The JSON pointers (RFC 6901) of the `input` values into which the engine injected a secret value; empty when none. Remove these values from logs and error messages.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


