# AgentDefinitionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_definition_key** | **models::AgentDefinitionKey** | The unique key for this agent definition. Unique across process definition versions.  | 
**agent_type** | [**models::AgentDefinitionTypeEnum**](AgentDefinitionTypeEnum.md) |  | 
**name** | **String** | The human-readable name of the process element that owns the agent definition. Falls back to elementId when the element has no BPMN name configured.  | 
**element_id** | **String** | The BPMN element ID of the process element that owns the agent definition. | 
**process_definition_id** | **String** | The BPMN process ID of the process definition that owns the agent definition. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition that owns the agent definition. | 
**process_definition_version** | **i32** | The version of the process definition that owns the agent definition. | 
**process_definition_version_tag** | Option<**String**> | The version tag of the process definition that owns the agent definition. | 
**tenant_id** | **String** | The tenant ID of this agent definition. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


