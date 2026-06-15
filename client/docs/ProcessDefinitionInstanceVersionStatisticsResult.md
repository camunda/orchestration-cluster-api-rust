# ProcessDefinitionInstanceVersionStatisticsResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | The ID associated with the process definition. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The unique key of the process definition. | 
**process_definition_name** | Option<**String**> | The name of the process definition. | 
**tenant_id** | **String** | The tenant ID associated with the process definition. | 
**process_definition_version** | **i32** | The version number of the process definition. | 
**active_instances_with_incident_count** | **i64** | The number of active process instances for this version that currently have incidents. | 
**active_instances_without_incident_count** | **i64** | The number of active process instances for this version that do not have any incidents. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


