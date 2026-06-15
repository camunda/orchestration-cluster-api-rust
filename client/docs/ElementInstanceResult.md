# ElementInstanceResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | The process definition ID associated to this element instance. | 
**start_date** | **chrono::DateTime<chrono::FixedOffset>** | Date when element instance started. | 
**end_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date when element instance finished. | 
**element_id** | **String** | The element ID for this element instance. | 
**element_name** | **String** | The element name for this element instance. | 
**r#type** | **Type** | Type of element as defined set of values. (enum: UNSPECIFIED, PROCESS, SUB_PROCESS, EVENT_SUB_PROCESS, AD_HOC_SUB_PROCESS, AD_HOC_SUB_PROCESS_INNER_INSTANCE, START_EVENT, INTERMEDIATE_CATCH_EVENT, INTERMEDIATE_THROW_EVENT, BOUNDARY_EVENT, END_EVENT, SERVICE_TASK, RECEIVE_TASK, USER_TASK, MANUAL_TASK, TASK, EXCLUSIVE_GATEWAY, INCLUSIVE_GATEWAY, PARALLEL_GATEWAY, EVENT_BASED_GATEWAY, SEQUENCE_FLOW, MULTI_INSTANCE_BODY, CALL_ACTIVITY, BUSINESS_RULE_TASK, SCRIPT_TASK, SEND_TASK, UNKNOWN) | 
**state** | [**models::ElementInstanceStateEnum**](ElementInstanceStateEnum.md) | State of element instance as defined set of values. | 
**has_incident** | **bool** | Shows whether this element instance has an incident. If true also an incidentKey is provided. | 
**tenant_id** | **String** | The tenant ID of the incident. | 
**element_instance_key** | **models::ElementInstanceKey** | The assigned key, which acts as a unique identifier for this element instance. | 
**process_instance_key** | **models::ProcessInstanceKey** | The process instance key associated to this element instance. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**process_definition_key** | **models::ProcessDefinitionKey** | The process definition key associated to this element instance. | 
**incident_key** | Option<**models::IncidentKey**> | Incident key associated with this element instance. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


