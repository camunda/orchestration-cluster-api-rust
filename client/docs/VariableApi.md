# \VariableApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_variable**](VariableApi.md#get_variable) | **GET** /variables/{variableKey} | Get variable
[**search_variables**](VariableApi.md#search_variables) | **POST** /variables/search | Search variables



## get_variable

> models::VariableResult get_variable(variable_key)
Get variable

Get a variable by its key.  This endpoint returns both process-level and local (element-scoped) variables. The variable's scopeKey indicates whether it's a process-level variable or scoped to a specific element instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**variable_key** | **VariableKey** | The variable key. | [required] |

### Return type

[**models::VariableResult**](VariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_variables

> models::VariableSearchQueryResult search_variables(truncate_values, variable_search_query)
Search variables

Search for variables based on given criteria.  This endpoint returns variables that exist directly at the specified scopes - it does not include variables from parent scopes that would be visible through the scope hierarchy.  Variables can be process-level (scoped to the process instance) or local (scoped to specific BPMN elements like tasks, subprocesses, etc.).  By default, long variable values in the response are truncated.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**truncate_values** | Option<**bool**> | When true (default), long variable values in the response are truncated. When false, full variable values are returned. |  |
**variable_search_query** | Option<[**VariableSearchQuery**](VariableSearchQuery.md)> |  |  |

### Return type

[**models::VariableSearchQueryResult**](VariableSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

