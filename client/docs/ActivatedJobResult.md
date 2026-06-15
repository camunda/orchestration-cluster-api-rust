# ActivatedJobResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** | The type of the job (should match what was requested). | 
**process_definition_id** | **String** | The bpmn process ID of the job's process definition. | 
**process_definition_version** | **i32** | The version of the job's process definition. | 
**element_id** | **String** | The associated task element ID. | 
**custom_headers** | **std::collections::HashMap<String, serde_json::Value>** | A set of custom headers defined during modelling; returned as a serialized JSON document. | 
**worker** | **String** | The name of the worker which activated this job. | 
**retries** | **i32** | The amount of retries left to this job (should always be positive). | 
**deadline** | **i64** | When the job can be activated again, sent as a UNIX epoch timestamp. | 
**variables** | **std::collections::HashMap<String, serde_json::Value>** | All variables visible to the task scope, computed at activation time. | 
**tenant_id** | **String** | The ID of the tenant that owns the job. | 
**job_key** | **models::JobKey** | The key, a unique identifier for the job. | 
**process_instance_key** | **models::ProcessInstanceKey** | The job's process instance key. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the job's process definition. | 
**element_instance_key** | **models::ElementInstanceKey** | The element instance key of the task. | 
**kind** | [**models::JobKindEnum**](JobKindEnum.md) |  | 
**listener_event_type** | [**models::JobListenerEventTypeEnum**](JobListenerEventTypeEnum.md) |  | 
**user_task** | Option<[**models::UserTaskProperties**](UserTaskProperties.md)> | User task properties, if the job is a user task. This is `null` if the job is not a user task.  | 
**tags** | **HashSet<String>** | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**priority** | **i32** | The priority of the job. Higher values indicate higher priority. Jobs created before 8.10 have no stored priority; the API returns 0 for such jobs.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


