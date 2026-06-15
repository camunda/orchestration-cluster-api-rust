# JobFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**deadline** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | When the job can next be activated. | [optional]
**denied_reason** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The reason provided by the user task listener for denying the work. | [optional]
**element_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The element ID associated with the job. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The element instance key associated with the job. | [optional]
**end_time** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | When the job ended. | [optional]
**error_code** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The error code provided for the failed job. | [optional]
**error_message** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The error message that provides additional context for a failed job. | [optional]
**has_failed_with_retries_left** | Option<**bool**> | Indicates whether the job has failed with retries left. | [optional]
**is_denied** | Option<**bool**> | Indicates whether the user task listener denies the work. | [optional]
**job_key** | Option<[**models::JobKeyFilterProperty**](JobKeyFilterProperty.md)> | The key, a unique identifier for the job. | [optional]
**kind** | Option<[**models::JobKindFilterProperty**](JobKindFilterProperty.md)> | The kind of the job. | [optional]
**listener_event_type** | Option<[**models::JobListenerEventTypeFilterProperty**](JobListenerEventTypeFilterProperty.md)> | The listener event type of the job. | [optional]
**priority** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The priority of the job. Jobs created before 8.10 have no stored priority and are excluded from results when this filter is applied.  | [optional]
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition ID associated with the job. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The process definition key associated with the job. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The process instance key associated with the job. | [optional]
**retries** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The number of retries left. | [optional]
**state** | Option<[**models::JobStateFilterProperty**](JobStateFilterProperty.md)> | The state of the job. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The tenant ID. | [optional]
**r#type** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The type of the job. | [optional]
**worker** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The name of the worker for this job. | [optional]
**creation_time** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | When the job was created. Field is present for jobs created after 8.9. | [optional]
**last_update_time** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | When the job was last updated. Field is present for jobs created after 8.9. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


