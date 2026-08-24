# ExpressionEvaluationResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**expression** | **String** | The evaluated expression | 
**result** | Option<**serde_json::Value**> | The result value. Its type can vary. | 
**warnings** | [**Vec<models::ExpressionEvaluationWarningItem>**](ExpressionEvaluationWarningItem.md) | List of warnings generated during expression evaluation | 
**referenced_secrets** | [**Vec<models::ExpressionSecretReferenceItem>**](ExpressionSecretReferenceItem.md) | The secret references resolved from trusted sources while evaluating the expression: a `camunda.secrets.<name>` reference used directly in the expression, or a reference carried by a `SECRET_REFERENCE`-kind cluster variable the expression read. References appearing only in request-body variables or plain cluster variables are excluded. Callers use this to know which `camunda.secrets.<name>` occurrences in the result they may safely resolve.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


