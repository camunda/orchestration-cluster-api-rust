# UsageMetricsResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_instances** | **i64** | The amount of created root process instances. | 
**decision_instances** | **i64** | The amount of executed decision instances. | 
**assignees** | **i64** | The amount of unique active task users. | 
**active_tenants** | **i64** | The amount of active tenants. | 
**tenants** | [**std::collections::HashMap<String, models::UsageMetricsResponseItem>**](UsageMetricsResponseItem.md) | The usage metrics by tenants. Only available if request `withTenants` query parameter was `true`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


