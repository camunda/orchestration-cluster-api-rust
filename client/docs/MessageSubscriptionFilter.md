# MessageSubscriptionFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**message_subscription_key** | Option<[**models::MessageSubscriptionKeyFilterProperty**](MessageSubscriptionKeyFilterProperty.md)> | The message subscription key associated with this message subscription. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The process definition key associated with this correlated message subscription. This only works for data created with 8.9 and later. | [optional]
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition ID associated with this message subscription. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The process instance key associated with this message subscription. | [optional]
**element_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The element ID associated with this message subscription. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The element instance key associated with this message subscription. | [optional]
**message_subscription_state** | Option<[**models::MessageSubscriptionStateFilterProperty**](MessageSubscriptionStateFilterProperty.md)> | The message subscription state. | [optional]
**last_updated_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The last updated date of the message subscription. | [optional]
**message_name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The name of the message associated with the message subscription. | [optional]
**correlation_key** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The correlation key of the message subscription. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The unique external tenant ID. | [optional]
**message_subscription_type** | Option<[**models::MessageSubscriptionTypeFilterProperty**](MessageSubscriptionTypeFilterProperty.md)> | The type of message subscription to filter by. When omitted, both `START_EVENT` and `PROCESS_EVENT` are returned. Only available for data created with Camunda 8.10 or later.  | [optional]
**process_definition_name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The name of the process definition associated with this message subscription. | [optional]
**process_definition_version** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The version of the process definition associated with this message subscription. | [optional]
**tool_name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Filter by tool name extracted from the `io.camunda.tool:name` zeebe:property. | [optional]
**inbound_connector_type** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Filter by inbound connector type extracted from the `inbound.type` zeebe:property. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


