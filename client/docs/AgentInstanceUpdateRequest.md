# AgentInstanceUpdateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_instance_key** | **models::ElementInstanceKey** | The key of the currently-active element instance for this agent instance. Used for ownership/equality validation against the stored agent instance and, when the supplied key differs from the previous association (re-entry of an ad-hoc sub-process or AI Agent task), appended to elementInstanceKeys with the reverse link updated on the supplied element instance.  | 
**status** | Option<[**models::AgentInstanceUpdateStatusEnum**](AgentInstanceUpdateStatusEnum.md)> | The new status of the agent instance. | [optional]
**metrics** | Option<[**models::AgentInstanceMetricsDelta**](AgentInstanceMetricsDelta.md)> | Metric increments to apply to the aggregate counters. | [optional]
**tools** | Option<[**Vec<models::AgentTool>**](AgentTool.md)> | The complete list of tools available to the agent, replacing any previously stored tools. When provided, the engine replaces the existing tool list with this value.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


