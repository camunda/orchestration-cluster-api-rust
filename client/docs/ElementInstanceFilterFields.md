# ElementInstanceFilterFields

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | Option<**String**> | The process definition ID associated to this element instance. | [optional]
**state** | Option<[**models::ElementInstanceStateFilterProperty**](ElementInstanceStateFilterProperty.md)> | State of element instance as defined set of values. | [optional]
**r#type** | Option<**Type**> | Type of element as defined set of values. (enum: UNSPECIFIED, PROCESS, SUB_PROCESS, EVENT_SUB_PROCESS, AD_HOC_SUB_PROCESS, AD_HOC_SUB_PROCESS_INNER_INSTANCE, START_EVENT, INTERMEDIATE_CATCH_EVENT, INTERMEDIATE_THROW_EVENT, BOUNDARY_EVENT, END_EVENT, SERVICE_TASK, RECEIVE_TASK, USER_TASK, MANUAL_TASK, TASK, EXCLUSIVE_GATEWAY, INCLUSIVE_GATEWAY, PARALLEL_GATEWAY, EVENT_BASED_GATEWAY, SEQUENCE_FLOW, MULTI_INSTANCE_BODY, CALL_ACTIVITY, BUSINESS_RULE_TASK, SCRIPT_TASK, SEND_TASK, UNKNOWN) | [optional]
**element_id** | Option<[**models::ElementIdFilterProperty**](ElementIdFilterProperty.md)> | The element ID for this element instance. | [optional]
**element_name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The element name. This only works for data created with 8.8 and onwards. Instances from prior versions don't contain this data and cannot be found.  | [optional]
**has_incident** | Option<**bool**> | Shows whether this element instance has an incident related to. | [optional]
**tenant_id** | Option<**String**> | The unique identifier of the tenant. | [optional]
**element_instance_key** | Option<**models::ElementInstanceKey**> | The assigned key, which acts as a unique identifier for this element instance. | [optional]
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The process instance key associated to this element instance. | [optional]
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | The process definition key associated to this element instance. | [optional]
**incident_key** | Option<**models::IncidentKey**> | The key of incident if field incident is true. | [optional]
**start_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The start date of this element instance. | [optional]
**end_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The end date of this element instance. | [optional]
**element_instance_scope_key** | Option<**String**> | The scope key of this element instance. If provided with a process instance key it will return element instances that are immediate children of the process instance. If provided with an element instance key it will return element instances that are immediate children of the element instance.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


