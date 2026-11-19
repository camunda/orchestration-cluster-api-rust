# ProcessDefinitionSuspensionVersionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition version. | 
**process_definition_version** | **i32** | The version of the process definition. | 
**outcome** | **Outcome** | `SUSPENDED` if the version was active and is now suspended. The suspension applies to all partitions shortly after the response. `ALREADY_SUSPENDED` if the version was already suspended, so its state is unchanged. With `includeProcessInstances`, its process instances are still suspended by the batch operation. `DRAINING` if it is being deleted and is left unchanged. This only occurs when the request selects by ID without a version; selecting a single draining version is rejected with 409.  (enum: SUSPENDED, ALREADY_SUSPENDED, DRAINING) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


