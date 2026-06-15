# MessageSubscriptionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**message_subscription_key** | **models::MessageSubscriptionKey** | The message subscription key associated with this message subscription. | 
**process_definition_id** | **String** | The process definition ID associated with this message subscription. | 
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | The process definition key associated with this message subscription. | 
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The process instance key associated with this message subscription. Only populated for intermediate event entities.  | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**element_id** | **String** | The element ID associated with this message subscription. | 
**element_instance_key** | Option<**models::ElementInstanceKey**> | The element instance key associated with this message subscription. Only populated for intermediate event entities.  | 
**message_subscription_state** | [**models::MessageSubscriptionStateEnum**](MessageSubscriptionStateEnum.md) |  | 
**last_updated_date** | **chrono::DateTime<chrono::FixedOffset>** | The last updated date of the message subscription. | 
**message_name** | **String** | The name of the message associated with the message subscription. | 
**correlation_key** | Option<**String**> | The correlation key of the message subscription. | 
**message_subscription_type** | [**models::MessageSubscriptionTypeEnum**](MessageSubscriptionTypeEnum.md) |  | 
**tool_properties** | **std::collections::HashMap<String, String>** | The subset of `zeebe:properties` extension properties whose keys start with the `io.camunda.tool:` prefix, extracted from the BPMN element associated with this subscription. Empty object when no matching properties are defined.  | 
**process_definition_name** | Option<**String**> | The name of the process definition associated with this message subscription. | 
**process_definition_version** | Option<**i32**> | The version of the process definition associated with this message subscription. | 
**tool_name** | Option<**String**> | Tool name extracted from the `io.camunda.tool:name` zeebe:property. Null when the property is absent.  | 
**inbound_connector_type** | Option<**String**> | Inbound connector type extracted from the `inbound.type` zeebe:property. Null when the property is absent.  | 
**tenant_id** | **String** | The unique identifier of the tenant. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


