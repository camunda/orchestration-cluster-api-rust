# VariableHistoryItemFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**variable_key** | Option<[**models::VariableKeyFilterProperty**](VariableKeyFilterProperty.md)> | The key of the variable this history item belongs to. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The key of the process instance of this variable. | [optional]
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Name of the variable. | [optional]
**scope_key** | Option<[**models::ScopeKeyFilterProperty**](ScopeKeyFilterProperty.md)> | The key of the scope that defines where this variable is directly defined. This can be a process instance key (for process-level variables) or an element instance key (for local variables scoped to tasks, sub-processes, gateways, events, etc.).  | [optional]
**element_id** | Option<[**models::ElementIdFilterProperty**](ElementIdFilterProperty.md)> | The model-defined id of the element that caused this change, if any. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The key of the element instance that caused this change, if any. | [optional]
**variable_history_item_key** | Option<[**models::VariableHistoryItemKeyFilterProperty**](VariableHistoryItemKeyFilterProperty.md)> | The unique key of this variable history item. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The key of the process definition. | [optional]
**process_definition_id** | Option<[**models::ProcessDefinitionIdFilterProperty**](ProcessDefinitionIdFilterProperty.md)> | The id of the process definition. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The tenant id of this variable. | [optional]
**timestamp** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The timestamp when this change occurred. | [optional]
**operation_type** | Option<[**models::VariableHistoryItemOperationTypeFilterProperty**](VariableHistoryItemOperationTypeFilterProperty.md)> | The type of operation that produced this variable change. | [optional]
**variable_operation_type** | Option<[**models::VariableHistoryItemVariableOperationTypeFilterProperty**](VariableHistoryItemVariableOperationTypeFilterProperty.md)> | What wrote this variable value. | [optional]
**actor_type** | Option<[**models::VariableHistoryItemActorTypeFilterProperty**](VariableHistoryItemActorTypeFilterProperty.md)> | The type of the actor who performed the operation. | [optional]
**actor_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The id of the user or client that caused this change, if known. | [optional]
**agent_element_id** | Option<[**models::ElementIdFilterProperty**](ElementIdFilterProperty.md)> | The element id of the agent that caused this change, for example the element id of an ad-hoc sub-process, if any.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


