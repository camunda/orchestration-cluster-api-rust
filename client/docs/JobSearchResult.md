# JobSearchResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**custom_headers** | **std::collections::HashMap<String, String>** | A set of custom headers defined during modelling. | 
**deadline** | Option<**chrono::DateTime<chrono::FixedOffset>**> | If the job has been activated, when it will next be available to be activated. | 
**denied_reason** | Option<**String**> | The reason provided by the user task listener for denying the work. | 
**element_id** | Option<**String**> | The element ID associated with the job. May be missing on job failure. | 
**element_instance_key** | **models::ElementInstanceKey** | The element instance key associated with the job. | 
**end_time** | Option<**chrono::DateTime<chrono::FixedOffset>**> | End date of the job. This is `null` if the job is not in an end state yet.  | 
**error_code** | Option<**String**> | The error code provided for a failed job. | 
**error_message** | Option<**String**> | The error message that provides additional context for a failed job. | 
**has_failed_with_retries_left** | **bool** | Indicates whether the job has failed with retries left. | 
**is_denied** | Option<**bool**> | Indicates whether the user task listener denies the work. | 
**job_key** | **models::JobKey** | The key, a unique identifier for the job. | 
**kind** | [**models::JobKindEnum**](JobKindEnum.md) |  | 
**listener_event_type** | [**models::JobListenerEventTypeEnum**](JobListenerEventTypeEnum.md) |  | 
**process_definition_id** | **String** | The process definition ID associated with the job. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The process definition key associated with the job. | 
**process_instance_key** | **models::ProcessInstanceKey** | The process instance key associated with the job. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**retries** | **i32** | The amount of retries left to this job. | 
**state** | [**models::JobStateEnum**](JobStateEnum.md) |  | 
**tenant_id** | **String** | The unique identifier of the tenant. | 
**r#type** | **String** | The type of the job. | 
**worker** | **String** | The name of the worker of this job. | 
**creation_time** | Option<**chrono::DateTime<chrono::FixedOffset>**> | When the job was created. Field is present for jobs created after 8.9. | 
**last_update_time** | Option<**chrono::DateTime<chrono::FixedOffset>**> | When the job was last updated. Field is present for jobs created after 8.9. | 
**priority** | **i32** | The priority of the job. Higher values indicate higher priority. Jobs created before 8.10 have no stored priority; they appear last when sorting by this field and are excluded when filtering by this field. The API returns 0 for such jobs.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


