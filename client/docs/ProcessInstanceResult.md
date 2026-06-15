# ProcessInstanceResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | Id of a process definition, from the model. Only ids of process definitions that are deployed are useful. | 
**process_definition_name** | Option<**String**> | The process definition name. | 
**process_definition_version** | **i32** | The process definition version. | 
**process_definition_version_tag** | Option<**String**> | The process definition version tag. | 
**start_date** | **chrono::DateTime<chrono::FixedOffset>** | The start time of the process instance. | 
**end_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The completion or termination time of the process instance. | 
**state** | [**models::ProcessInstanceStateEnum**](ProcessInstanceStateEnum.md) |  | 
**has_incident** | **bool** | Whether this process instance has a related incident or not. | 
**tenant_id** | **String** | The unique identifier of the tenant. | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of this process instance. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The process definition key. | 
**parent_process_instance_key** | Option<**models::ProcessInstanceKey**> | The parent process instance key. | 
**parent_element_instance_key** | Option<**models::ElementInstanceKey**> | The parent element instance key. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**tags** | **HashSet<String>** | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | 
**business_id** | Option<**String**> | The business id associated with this process instance. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


