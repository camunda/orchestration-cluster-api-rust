# CorrelatedMessageSubscriptionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**correlation_key** | Option<**String**> | The correlation key of the message. | 
**correlation_time** | **chrono::DateTime<chrono::FixedOffset>** | The time when the message was correlated. | 
**element_id** | **String** | The element ID that received the message. | 
**element_instance_key** | Option<**models::ElementInstanceKey**> | The element instance key that received the message. It is `null` for start event subscriptions.  | 
**message_key** | **models::MessageKey** | The message key. | 
**message_name** | **String** | The name of the message. | 
**partition_id** | **i32** | The partition ID that correlated the message. | 
**process_definition_id** | **String** | The process definition ID associated with this correlated message subscription. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The process definition key associated with this correlated message subscription. | 
**process_instance_key** | **models::ProcessInstanceKey** | The process instance key associated with this correlated message subscription. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**subscription_key** | **models::MessageSubscriptionKey** | The subscription key that received the message. | 
**tenant_id** | **String** | The tenant ID associated with this correlated message subscription. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


