# AgentInstanceResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_instance_key** | **models::AgentInstanceKey** | The unique key for this agent instance. | 
**status** | [**models::AgentInstanceStatusEnum**](AgentInstanceStatusEnum.md) |  | 
**definition** | [**models::AgentInstanceDefinition**](AgentInstanceDefinition.md) | The static definition of the agent, including model, provider, and system prompt. | 
**metrics** | [**models::AgentInstanceMetrics**](AgentInstanceMetrics.md) | Aggregated metrics across all loopIterations of this agent instance. | 
**limits** | [**models::AgentInstanceLimits**](AgentInstanceLimits.md) | The configured limits for this agent instance, set once at creation. | 
**tools** | [**Vec<models::AgentTool>**](AgentTool.md) | The tools available to the agent. | 
**element_id** | **String** | The BPMN element ID of the ad-hoc sub-process or AI agent task that owns this agent instance. | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of the process instance that owns this agent instance. | 
**root_process_instance_key** | **models::ProcessInstanceKey** | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy.  | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition associated with this agent instance. | 
**process_definition_id** | **String** | The BPMN process ID of the process definition associated with this agent instance. | 
**process_definition_version** | **i32** | The version of the process definition associated with this agent instance. | 
**process_definition_version_tag** | Option<**String**> | The version tag of the process definition associated with this agent instance. | 
**tenant_id** | **String** | The tenant ID of this agent instance. | 
**creation_date** | **chrono::DateTime<chrono::FixedOffset>** | The date when this agent instance was created. | 
**last_updated_date** | **chrono::DateTime<chrono::FixedOffset>** | The date when this agent instance was last updated. | 
**completion_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The date when this agent instance completed. Null while the agent is still running. | 
**element_instance_keys** | **Vec<String>** | The keys of all element instances associated with this agent instance. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


