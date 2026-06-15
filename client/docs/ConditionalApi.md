# \ConditionalApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**evaluate_conditionals**](ConditionalApi.md#evaluate_conditionals) | **POST** /conditionals/evaluation | Evaluate root level conditional start events



## evaluate_conditionals

> models::EvaluateConditionalResult evaluate_conditionals(conditional_evaluation_instruction)
Evaluate root level conditional start events

Evaluates root-level conditional start events for process definitions. If the evaluation is successful, it will return the keys of all created process instances, along with their associated process definition key. Multiple root-level conditional start events of the same process definition can trigger if their conditions evaluate to true. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**conditional_evaluation_instruction** | [**ConditionalEvaluationInstruction**](ConditionalEvaluationInstruction.md) |  | [required] |

### Return type

[**models::EvaluateConditionalResult**](EvaluateConditionalResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

