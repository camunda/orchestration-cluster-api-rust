# CorrelatedMessageSubscriptionFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**business_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Filter by the business id stored on the correlated message subscription — for message start event correlations the correlating message's business id, and for catch, boundary, or intermediate event correlations the subscribing process instance's business id. Supports advanced string filtering, including `$like` with `*`/`?` wildcards.  | [optional]
**correlation_key** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The correlation key of the message. | [optional]
**correlation_time** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The time when the message was correlated. | [optional]
**element_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The element ID that received the message. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The element instance key that received the message. | [optional]
**message_key** | Option<[**models::BasicStringFilterProperty**](BasicStringFilterProperty.md)> | The message key. | [optional]
**message_name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The name of the message. | [optional]
**partition_id** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The partition ID that correlated the message. | [optional]
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition ID associated with this correlated message subscription. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The process definition key associated with this correlated message subscription. For intermediate message events, this only works for data created with 8.9 and later. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The process instance key associated with this correlated message subscription. | [optional]
**subscription_key** | Option<[**models::MessageSubscriptionKeyFilterProperty**](MessageSubscriptionKeyFilterProperty.md)> | The subscription key that received the message. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The tenant ID associated with this correlated message subscription. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


