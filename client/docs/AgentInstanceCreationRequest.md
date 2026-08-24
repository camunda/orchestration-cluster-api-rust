# AgentInstanceCreationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_instance_key** | **models::ElementInstanceKey** | The key of the AI Agent Sub-process or AI Agent Task element instance. The engine uses this key to infer processInstanceKey, elementId, processDefinitionKey, and tenantId.  | 
**definition** | Option<[**models::AgentInstanceDefinition**](AgentInstanceDefinition.md)> | The agent's initial definition; model, provider, and systemPrompt can all be changed later via a CONFIGURATION history item. Required when history is empty or omitted. Must be omitted when history is non-empty — supply model, provider, and systemPrompt through a CONFIGURATION item in history instead.  | [optional]
**limits** | Option<[**models::AgentInstanceLimits**](AgentInstanceLimits.md)> | Limits for the agent execution. When omitted, all limits default to -1 (no limit). Must be omitted when history is non-empty — supply limits through a CONFIGURATION item in history instead, if needed.  | [optional]
**job_key** | Option<**models::JobKey**> | The key of the job activation during which this creation is being made. Required whenever history is non-empty.  | [optional]
**job_lease** | Option<**String**> | Opaque lease token received from the job activation response. Disambiguates this activation from any other activation of the same job: if the job is later retried, history items submitted under a superseded lease are discarded rather than committed.  | [optional]
**history** | Option<[**Vec<models::AgentInstanceHistoryItem>**](AgentInstanceHistoryItem.md)> | A batch of history items to append to the agent instance's conversation history, in request order. Each created item is echoed back in the response's createdHistory, positionally correlated. When non-empty, model, provider, and systemPrompt (and, if needed, limits) must be established through a CONFIGURATION item in this batch instead of the top-level definition/limits, which must then be omitted.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


