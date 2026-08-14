# ClusterModeChangeResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**change_id** | **String** | The ID of the cluster change that was triggered by the request. | 
**planned_changes** | [**Vec<models::ClusterModeChangePlannedChange>**](ClusterModeChangePlannedChange.md) | The operations that will be applied to complete the change, grouped by the physical tenant they belong to. Groups are transitioned in parallel; the operations within a group are applied in the given order. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


