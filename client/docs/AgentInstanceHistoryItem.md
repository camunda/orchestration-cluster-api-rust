# AgentInstanceHistoryItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**history_item_id** | **String** | Caller-assigned identifier used to detect and dedupe retries of the same item. For example, when a retried job activation resubmits history items it already sent in an earlier attempt, those items are not rejected; they are flagged via isDuplicate in the response instead. Must be non-blank.  | 
**loop_iteration** | **i32** | The loop iteration this item belongs to. | 
**role** | [**models::AgentInstanceHistoryRoleEnum**](AgentInstanceHistoryRoleEnum.md) | The role of this history item in the conversation. | 
**content** | [**Vec<models::AgentInstanceMessageContent>**](AgentInstanceMessageContent.md) | The content blocks of this history item. | 
**tool_calls** | Option<[**Vec<models::AgentInstanceToolCall>**](AgentInstanceToolCall.md)> | Tool calls associated with this history item. For ASSISTANT items: tool calls dispatched by this LLM response. For TOOL_RESULT items: single-entry array referencing the originating tool call. Omit for USER items.  | [optional]
**metrics** | Option<[**models::AgentInstanceHistoryItemMetrics**](AgentInstanceHistoryItemMetrics.md)> | Per-call token and latency metrics. Present on ASSISTANT items only. | [optional]
**produced_at** | **chrono::DateTime<chrono::FixedOffset>** | The agent-side timestamp of when this message was produced. | 
**tools** | Option<[**Vec<models::AgentTool>**](AgentTool.md)> | The complete list of tools available to the agent as of this entry. CONFIGURATION items only; omit for other roles. Omit to leave the tool list unchanged; send an empty array to clear it.  | [optional]
**model** | Option<**String**> | The LLM model identifier as of this entry. CONFIGURATION items only; omit for other roles.  | [optional]
**provider** | Option<**String**> | The LLM provider as of this entry. CONFIGURATION items only; omit for other roles.  | [optional]
**limits** | Option<[**models::AgentInstanceLimits**](AgentInstanceLimits.md)> | The operational limits as of this entry. CONFIGURATION items only; omit for other roles.  | [optional]
**system_prompt** | Option<[**Vec<models::AgentInstanceMessageContent>**](AgentInstanceMessageContent.md)> | The system prompt, as content blocks, as of this entry. CONFIGURATION items only; omit for other roles. Omit to leave the system prompt unchanged; when present, must be non-empty.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


