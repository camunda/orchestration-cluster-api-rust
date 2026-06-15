# ConditionalEvaluationInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tenant_id** | Option<**String**> | Used to evaluate root-level conditional start events for a tenant with the given ID. This will only evaluate root-level conditional start events of process definitions which belong to the tenant.  | [optional]
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | Used to evaluate root-level conditional start events of the process definition with the given key.  | [optional]
**variables** | **std::collections::HashMap<String, serde_json::Value>** | JSON object representing the variables to use for evaluation of the conditions and to pass to the process instances that have been triggered.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


