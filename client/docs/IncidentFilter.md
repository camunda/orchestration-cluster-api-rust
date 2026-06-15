# IncidentFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition ID associated to this incident. | [optional]
**error_type** | Option<[**models::IncidentErrorTypeFilterProperty**](IncidentErrorTypeFilterProperty.md)> | Incident error type with a defined set of values. | [optional]
**error_message** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The error message of this incident. | [optional]
**element_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The element ID associated to this incident. | [optional]
**creation_time** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | Date of incident creation. | [optional]
**state** | Option<[**models::IncidentStateFilterProperty**](IncidentStateFilterProperty.md)> | State of this incident with a defined set of values. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The tenant ID of the incident. | [optional]
**incident_key** | Option<[**models::BasicStringFilterProperty**](BasicStringFilterProperty.md)> | The assigned key, which acts as a unique identifier for this incident. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The process definition key associated to this incident. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The process instance key associated to this incident. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The element instance key associated to this incident. | [optional]
**job_key** | Option<[**models::JobKeyFilterProperty**](JobKeyFilterProperty.md)> | The job key, if exists, associated with this incident. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


