# DeleteResourceResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**resource_key** | [**models::ResourceKey**](ResourceKey.md) | The system-assigned key for this resource, requested to be deleted. | 
**batch_operation** | Option<[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)> | The batch operation created for asynchronously deleting the historic data.  This field is only populated when the request `deleteHistory` is set to `true` and the resource is a process definition. For other resource types (decisions, forms, generic resources), this field will be `null`.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


