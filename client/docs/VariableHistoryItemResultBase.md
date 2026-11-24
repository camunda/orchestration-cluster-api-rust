# VariableHistoryItemResultBase

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**variable_history_item_key** | **models::VariableHistoryItemKey** | The unique key of this variable history item. | 
**variable_key** | **models::VariableKey** | The key of the variable this history item belongs to. | 
**name** | **String** | Name of the variable. | 
**scope_key** | [**models::ScopeKey**](ScopeKey.md) | The key of the scope where this variable is directly defined. For process-level variables, this is the process instance key. For local variables, this is the key of the specific element instance where the variable is directly defined.  | 
**element_instance_key** | Option<**models::ElementInstanceKey**> | The key of the element instance that caused this change, if any. | 
**element_id** | Option<**String**> | The model-defined id of the element that caused this change, if any. | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of the process instance of this variable. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition. | 
**process_definition_id** | **String** | The id of the process definition. | 
**tenant_id** | **String** | The tenant id of this variable. | 
**timestamp** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when this change occurred. | 
**operation_type** | [**models::VariableHistoryItemOperationTypeEnum**](VariableHistoryItemOperationTypeEnum.md) |  | 
**variable_operation_type** | [**models::VariableHistoryItemVariableOperationTypeEnum**](VariableHistoryItemVariableOperationTypeEnum.md) |  | 
**actor_type** | [**models::VariableHistoryItemActorTypeEnum**](VariableHistoryItemActorTypeEnum.md) |  | 
**actor_id** | Option<**String**> | The id of the user or client that caused this change, if known. | 
**agent_element_id** | Option<**String**> | The element id of the agent that caused this change, for example the element id of an ad-hoc sub-process, if any.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


