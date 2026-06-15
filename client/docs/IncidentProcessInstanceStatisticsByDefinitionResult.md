# IncidentProcessInstanceStatisticsByDefinitionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | Id of a process definition, from the model. Only ids of process definitions that are deployed are useful. | 
**process_definition_key** | **models::ProcessDefinitionKey** |  | 
**process_definition_name** | **String** | The name of the process definition. | 
**process_definition_version** | **i32** | The version of the process definition. | 
**tenant_id** | **String** | The unique identifier of the tenant. | 
**active_instances_with_error_count** | **i64** | The number of active process instances that currently have an incident with the specified error hash code.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


