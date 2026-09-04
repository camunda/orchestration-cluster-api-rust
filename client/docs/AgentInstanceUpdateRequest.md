# AgentInstanceUpdateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_instance_key** | **models::ElementInstanceKey** | The key of the currently-active element instance for this agent instance. Used for ownership/equality validation against the stored agent instance and, when the supplied key differs from the previous association (re-entry of an ad-hoc sub-process or AI Agent task), appended to elementInstanceKeys with the reverse link updated on the supplied element instance. Only one element instance may hold this write claim at a time: any update from a different element instance is rejected while the current writer's job is still active.  | 
**status** | Option<[**models::AgentInstanceUpdateStatusEnum**](AgentInstanceUpdateStatusEnum.md)> | The new status of the agent instance. | [optional]
**job_key** | **models::JobKey** | The key of the job activation during which this update is being made. An update must always be attributed to the active job that produced it.  | 
**job_lease** | **String** | Opaque lease token received from the job activation response. Disambiguates this activation from any other activation of the same job: if the job is later retried, history items submitted under a superseded lease are discarded rather than committed.  | 
**history** | Option<[**Vec<models::AgentInstanceHistoryItem>**](AgentInstanceHistoryItem.md)> | A batch of history items to append to the agent instance's conversation history, in request order. Each created item is echoed back in the response's createdHistory, positionally correlated.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


