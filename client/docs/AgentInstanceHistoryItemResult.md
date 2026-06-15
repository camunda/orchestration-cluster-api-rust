# AgentInstanceHistoryItemResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**history_item_key** | **models::AgentHistoryItemKey** | The unique key for this history item. Stable and sortable by creation order. | 
**agent_instance_key** | **models::AgentInstanceKey** | The key of the agent instance this item belongs to. | 
**element_instance_key** | **models::ElementInstanceKey** | The key of the AI Agent Task or ad-hoc sub-process element instance under which this item was produced. | 
**job_key** | **models::JobKey** | The key of the job activation during which this item was produced. | 
**job_lease** | **String** | The lease token of the activation that produced this item. | 
**iteration** | Option<**i32**> | The sequential iteration number this item belongs to. Null if not provided by the connector. | 
**role** | [**models::AgentInstanceHistoryRoleEnum**](AgentInstanceHistoryRoleEnum.md) | The role of this history item in the conversation. | 
**content** | [**Vec<models::AgentInstanceMessageContent>**](AgentInstanceMessageContent.md) | The content blocks of this history item. | 
**tool_calls** | [**Vec<models::AgentInstanceToolCall>**](AgentInstanceToolCall.md) | Tool calls for this item. Empty for USER items and ASSISTANT items with no tool dispatches. ASSISTANT items: dispatched tool calls with arguments populated. TOOL_RESULT items: single-entry array referencing the originating tool call (arguments null).  | 
**metrics** | Option<[**models::AgentInstanceHistoryItemMetrics**](AgentInstanceHistoryItemMetrics.md)> | Per-call token and latency metrics. Present on ASSISTANT items only. | 
**commit_status** | [**models::AgentInstanceHistoryCommitStatusEnum**](AgentInstanceHistoryCommitStatusEnum.md) | The commit status of this history item. | 
**produced_at** | **chrono::DateTime<chrono::FixedOffset>** | The connector-side timestamp of when this message was produced. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


