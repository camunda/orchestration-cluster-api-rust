# AgentInstanceToolCall

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tool_call_id** | **String** | The LLM-assigned tool call ID. Correlates ASSISTANT items to their matching TOOL_RESULT items. | 
**tool_name** | **String** | The LLM-visible tool name. | 
**element_id** | Option<**String**> | The BPMN element ID handling this tool. | 
**arguments** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The tool call arguments as provided by the LLM. Null on TOOL_RESULT items. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


