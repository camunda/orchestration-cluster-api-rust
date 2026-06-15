# AgentInstanceFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_instance_key** | Option<[**models::AgentInstanceKeyFilterProperty**](AgentInstanceKeyFilterProperty.md)> | The unique key of the agent instance. | [optional]
**status** | Option<[**models::AgentInstanceStatusFilterProperty**](AgentInstanceStatusFilterProperty.md)> | The current status of the agent instance. | [optional]
**element_id** | Option<[**models::ElementIdFilterProperty**](ElementIdFilterProperty.md)> | The BPMN element ID of the agent task. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The key of the process instance that owns this agent instance. | [optional]
**root_process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The key of the root process instance. Filters agent instances belonging to a specific call hierarchy. The root process instance is the top-level ancestor in the process instance hierarchy.  | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The key of the process definition associated with this agent instance. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The tenant ID of the agent instance. | [optional]
**creation_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The creation date of the agent instance. | [optional]
**last_updated_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The date the agent instance was last updated. | [optional]
**completion_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The completion date of the agent instance. | [optional]
**element_instance_keys** | Option<[**Vec<models::ElementInstanceKeyFilterProperty>**](ElementInstanceKeyFilterProperty.md)> | The keys of element instances associated with this agent instance. If multiple keys are provided, the filter matches agent instances associated with all of the provided keys at the same time. | [optional]
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The BPMN process ID of the process definition associated with this agent instance. | [optional]
**process_definition_version** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The version of the process definition associated with this agent instance. | [optional]
**process_definition_version_tag** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The version tag of the process definition associated with this agent instance. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


