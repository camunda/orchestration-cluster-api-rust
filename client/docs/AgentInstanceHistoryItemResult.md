# AgentInstanceHistoryItemResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**history_item_key** | **models::AgentHistoryItemKey** | The unique key for this history item. Stable and sortable by creation order. | 
**history_item_id** | **String** | The client-supplied identifier this item was created with. Empty for items that don't carry one.  | 
**agent_instance_key** | **models::AgentInstanceKey** | The key of the agent instance this item belongs to. | 
**element_instance_key** | **models::ElementInstanceKey** | The key of the AI Agent Task or ad-hoc sub-process element instance under which this item was produced. | 
**job_key** | **models::JobKey** | The key of the job activation during which this item was produced. | 
**job_lease** | **String** | The lease token of the activation that produced this item. | 
**loop_iteration** | **i32** | The loop iteration this item belongs to. | 
**role** | [**models::AgentInstanceHistoryRoleEnum**](AgentInstanceHistoryRoleEnum.md) | The role of this history item in the conversation. | 
**content** | [**Vec<models::AgentInstanceMessageContent>**](AgentInstanceMessageContent.md) | The content blocks of this history item. | 
**tool_calls** | [**Vec<models::AgentInstanceToolCall>**](AgentInstanceToolCall.md) | Tool calls for this item. Empty for USER items and ASSISTANT items with no tool dispatches. ASSISTANT items: dispatched tool calls. TOOL_RESULT items: single-entry array referencing the originating tool call.  | 
**metrics** | Option<[**models::AgentInstanceHistoryItemMetrics**](AgentInstanceHistoryItemMetrics.md)> | Per-call token and latency metrics. Null when metrics were not provided at creation time. | 
**commit_status** | [**models::AgentInstanceHistoryCommitStatusEnum**](AgentInstanceHistoryCommitStatusEnum.md) | The commit status of this history item. | 
**produced_at** | **chrono::DateTime<chrono::FixedOffset>** | The agent-side timestamp of when this message was produced. | 
**tools** | [**Vec<models::AgentTool>**](AgentTool.md) | The complete list of tools available to the agent as of this entry. CONFIGURATION items only; empty for other roles.  | 
**model** | Option<**String**> | The LLM model identifier as of this entry. CONFIGURATION items only; null for other roles.  | 
**provider** | Option<**String**> | The LLM provider as of this entry. CONFIGURATION items only; null for other roles.  | 
**limits** | [**models::AgentInstanceLimits**](AgentInstanceLimits.md) | The operational limits as of this entry. CONFIGURATION items only; -1 on any field means \"no limit configured\" for other roles.  | 
**system_prompt** | [**Vec<models::AgentInstanceMessageContent>**](AgentInstanceMessageContent.md) | The system prompt, as content blocks, as of this entry. CONFIGURATION items only; empty for other roles.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


