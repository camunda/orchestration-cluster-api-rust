# ProcessDefinitionInstanceStatisticsResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | Id of a process definition, from the model. Only ids of process definitions that are deployed are useful. | 
**tenant_id** | **String** | The unique identifier of the tenant. | 
**latest_process_definition_name** | Option<**String**> | Name of the latest deployed process definition instance version. | 
**has_multiple_versions** | **bool** | Indicates whether multiple versions of this process definition instance are deployed. | 
**active_instances_without_incident_count** | **i64** | Total number of currently active process instances of this definition that do not have incidents. | 
**active_instances_with_incident_count** | **i64** | Total number of currently active process instances of this definition that have at least one incident. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


