# ProbeActivationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**probe_ids** | **Vec<String>** | The probe IDs to collect probes for: 1 to 100 distinct values, each 1 to 256 characters, without whitespace, and not `*`. Probe IDs on which the caller lacks the `ANSWER` permission are skipped.  | 
**worker** | Option<**String**> | The name of the worker, returned to the client in the answer. | [optional]
**max_probes_to_activate** | **i32** | The maximum number of probes to collect with this request, at least 1. | 
**request_timeout** | Option<**i64**> | How long to hold the request until a probe is collected, in milliseconds. With `0` or absent, the cluster's long polling timeout is used. With a value below `0`, long polling is off and the request returns at once, also when it collected no probe.  | [optional]
**tenant_ids** | Option<**Vec<String>**> | A list of IDs of tenants for which to collect probes. | [optional]
**tenant_filter** | Option<[**models::TenantFilterEnum**](TenantFilterEnum.md)> | The tenant filtering strategy - determines whether to use provided tenant IDs or assigned tenant IDs from the authenticated principal's authorized tenants.  | [optional][default to Provided]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


