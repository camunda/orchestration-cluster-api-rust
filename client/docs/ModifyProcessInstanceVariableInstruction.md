# ModifyProcessInstanceVariableInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**variables** | **std::collections::HashMap<String, serde_json::Value>** | JSON document that will instantiate the variables at the scope defined by the scopeId. It must be a JSON object, as variables will be mapped in a key-value fashion.  | 
**scope_id** | Option<**String**> | The id of the element in which scope the variables should be created. Leave empty to create the variables in the global scope of the process instance.  | [optional][default to ]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


