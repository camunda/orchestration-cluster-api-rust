# ClusterVariableResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of the cluster variable. Unique within its scope (global or tenant-specific). | 
**scope** | [**models::ClusterVariableScopeEnum**](ClusterVariableScopeEnum.md) |  | 
**tenant_id** | Option<**String**> | Only provided if the cluster variable scope is TENANT. Null for global scope variables. | 
**value** | **String** | Full value of this cluster variable. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


