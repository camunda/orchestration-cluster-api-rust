# ClusterRestorePlannedChange

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant the operations apply to; null for operations that are not scoped to a single physical tenant, such as broker lifecycle operations. | 
**operations** | [**Vec<models::ClusterRestoreOperation>**](ClusterRestoreOperation.md) | The ordered list of operations that will be applied to the physical tenant. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


