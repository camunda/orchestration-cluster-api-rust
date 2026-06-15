# AgentInstanceCreationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_instance_key** | **models::ElementInstanceKey** | The key of the AHSP or AI Agent Task element instance. The engine uses this key to infer processInstanceKey, elementId, processDefinitionKey, and tenantId.  | 
**definition** | [**models::AgentInstanceDefinition**](AgentInstanceDefinition.md) | Static definition set once at creation. | 
**limits** | Option<[**models::AgentInstanceLimits**](AgentInstanceLimits.md)> | Limits for the agent execution. When omitted, all limits default to -1 (no limit).  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


