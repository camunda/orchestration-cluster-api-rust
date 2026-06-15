# ExpressionEvaluationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**expression** | **String** | The expression to evaluate (e.g., \"=x + y\") | 
**tenant_id** | Option<**String**> | Required when the expression references tenant-scoped cluster variables | [optional]
**scope_key** | Option<[**models::ScopeKey**](ScopeKey.md)> | Key of the process instance or element instance whose variables should be made visible to the expression. Use a process instance key to evaluate against the process instance scope, or an element instance key to evaluate against that element instance scope. If omitted, the expression is evaluated unscoped, using only cluster variables and request-body variables.  | [optional]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Optional variables for expression evaluation. These variables are only used for the current evaluation and do not persist beyond it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


