# AgentInstanceUpdateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_instance_key** | **models::ElementInstanceKey** | The key of the currently-active element instance for this agent instance. Used for ownership/equality validation against the stored agent instance and, when the supplied key differs from the previous association (re-entry of an ad-hoc sub-process or AI Agent task), appended to elementInstanceKeys with the reverse link updated on the supplied element instance.  | 
**status** | Option<[**models::AgentInstanceUpdateStatusEnum**](AgentInstanceUpdateStatusEnum.md)> | The new status of the agent instance. | [optional]
**metrics** | Option<[**models::AgentInstanceMetricsDelta**](AgentInstanceMetricsDelta.md)> | Metric increments to apply to the aggregate counters. | [optional]
**tools** | Option<[**Vec<models::AgentTool>**](AgentTool.md)> | The complete list of tools available to the agent, replacing any previously stored tools. When provided, the engine replaces the existing tool list with this value.  | [optional]
**job_key** | Option<**models::JobKey**> | The key of the job activation during which this update is being made. Required whenever history is provided.  | [optional]
**job_lease** | Option<**String**> | Opaque lease token received from the job activation response. Disambiguates this activation from any other activation of the same job: if the job is later retried, history items submitted under a superseded lease are discarded rather than committed.  | [optional]
**history** | Option<[**Vec<models::AgentInstanceHistoryItem>**](AgentInstanceHistoryItem.md)> | A batch of history items to append to the agent instance's conversation history, in request order. Each created item is echoed back in the response's createdHistory, positionally correlated.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


