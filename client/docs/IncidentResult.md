# IncidentResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | The process definition ID associated to this incident. | 
**error_type** | [**models::IncidentErrorTypeEnum**](IncidentErrorTypeEnum.md) | The type of the incident error. | 
**error_message** | **String** | Error message which describes the error in more detail. | 
**element_id** | **String** | The element ID associated to this incident. | 
**creation_time** | **chrono::DateTime<chrono::FixedOffset>** | The creation time of the incident. | 
**state** | [**models::IncidentStateEnum**](IncidentStateEnum.md) | The incident state. | 
**tenant_id** | **String** | The tenant ID of the incident. | 
**incident_key** | **models::IncidentKey** | The assigned key, which acts as a unique identifier for this incident. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The process definition key associated to this incident. | 
**process_instance_key** | **models::ProcessInstanceKey** | The process instance key associated to this incident. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**element_instance_key** | **models::ElementInstanceKey** | The element instance key associated to this incident. | 
**job_key** | Option<**models::JobKey**> | The job key, if exists, associated with this incident. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


