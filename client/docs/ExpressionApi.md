# \ExpressionApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**evaluate_expression**](ExpressionApi.md#evaluate_expression) | **POST** /expression/evaluation | Evaluate an expression



## evaluate_expression

> models::ExpressionEvaluationResult evaluate_expression(expression_evaluation_request)
Evaluate an expression

Evaluates a FEEL expression and returns the result. Supports references to tenant scoped cluster variables when a tenant ID is provided. Optionally, provide a `scopeKey` to make the variables of a specific process instance or element instance visible while evaluating the expression. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**expression_evaluation_request** | [**ExpressionEvaluationRequest**](ExpressionEvaluationRequest.md) |  | [required] |

### Return type

[**models::ExpressionEvaluationResult**](ExpressionEvaluationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

