# DeleteResourceResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**resource_key** | [**models::ResourceKey**](ResourceKey.md) | The system-assigned key for this resource, requested to be deleted. | 
**batch_operation** | Option<[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)> | The batch operation created for asynchronously deleting the historic data.  Populated when `deleteHistory` is `true` and either the resource is a decision requirements definition, or the resource is a process definition that is already fully deleted from the runtime state (its history is purged directly by a batch operation).  For a process definition that still exists in the runtime state, deletion first drains the definition and its history is removed asynchronously as part of that lifecycle, so no batch operation is returned and this field is `null`. It is also `null` for forms and generic resources.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


