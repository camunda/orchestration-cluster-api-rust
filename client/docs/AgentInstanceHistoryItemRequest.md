# AgentInstanceHistoryItemRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_instance_key** | **models::ElementInstanceKey** | The key of the currently-active element instance.  | 
**job_key** | **models::JobKey** | The key of the current job activation during which this history item was produced. | 
**job_lease** | **String** | Opaque lease token received from the job activation response. | 
**iteration** | Option<**i32**> | Sequential iteration number this item belongs to. Omit if not grouping items into iterations. | [optional]
**role** | [**models::AgentInstanceHistoryRoleEnum**](AgentInstanceHistoryRoleEnum.md) | The role of this history item in the conversation. | 
**content** | [**Vec<models::AgentInstanceMessageContent>**](AgentInstanceMessageContent.md) | The content blocks of this history item. | 
**tool_calls** | Option<[**Vec<models::AgentInstanceToolCall>**](AgentInstanceToolCall.md)> | Tool calls associated with this history item. For ASSISTANT items: tool calls dispatched by this LLM response, with arguments populated. For TOOL_RESULT items: single-entry array referencing the originating tool call, with arguments null. Omit for USER items.  | [optional]
**metrics** | Option<[**models::AgentInstanceHistoryItemMetrics**](AgentInstanceHistoryItemMetrics.md)> | Per-call token and latency metrics. Present on ASSISTANT items only. | [optional]
**produced_at** | **chrono::DateTime<chrono::FixedOffset>** | The connector-side timestamp of when this message was produced. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


