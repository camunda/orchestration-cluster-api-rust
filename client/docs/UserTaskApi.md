# \UserTaskApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assign_user_task**](UserTaskApi.md#assign_user_task) | **POST** /user-tasks/{userTaskKey}/assignment | Assign user task
[**complete_user_task**](UserTaskApi.md#complete_user_task) | **POST** /user-tasks/{userTaskKey}/completion | Complete user task
[**get_user_task**](UserTaskApi.md#get_user_task) | **GET** /user-tasks/{userTaskKey} | Get user task
[**get_user_task_form**](UserTaskApi.md#get_user_task_form) | **GET** /user-tasks/{userTaskKey}/form | Get user task form
[**search_user_task_audit_logs**](UserTaskApi.md#search_user_task_audit_logs) | **POST** /user-tasks/{userTaskKey}/audit-logs/search | Search user task audit logs
[**search_user_task_effective_variables**](UserTaskApi.md#search_user_task_effective_variables) | **POST** /user-tasks/{userTaskKey}/effective-variables/search | Search user task effective variables
[**search_user_task_variables**](UserTaskApi.md#search_user_task_variables) | **POST** /user-tasks/{userTaskKey}/variables/search | Search user task variables
[**search_user_tasks**](UserTaskApi.md#search_user_tasks) | **POST** /user-tasks/search | Search user tasks
[**unassign_user_task**](UserTaskApi.md#unassign_user_task) | **DELETE** /user-tasks/{userTaskKey}/assignee | Unassign user task
[**update_user_task**](UserTaskApi.md#update_user_task) | **PATCH** /user-tasks/{userTaskKey} | Update user task



## assign_user_task

> assign_user_task(user_task_key, user_task_assignment_request)
Assign user task

Assigns a user task with the given key to the given assignee. Assignment waits for blocking task listeners on this lifecycle transition. If listener processing is delayed beyond the request timeout, this endpoint can return 504. Other gateway timeout causes are also possible. Retry with backoff and inspect listener worker availability and logs when this repeats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task to assign. | [required] |
**user_task_assignment_request** | [**UserTaskAssignmentRequest**](UserTaskAssignmentRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## complete_user_task

> complete_user_task(user_task_key, user_task_completion_request)
Complete user task

Completes a user task with the given key. Completion waits for blocking task listeners on this lifecycle transition. If listener processing is delayed beyond the request timeout, this endpoint can return 504. Other gateway timeout causes are also possible. Retry with backoff and inspect listener worker availability and logs when this repeats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task to complete. | [required] |
**user_task_completion_request** | Option<[**UserTaskCompletionRequest**](UserTaskCompletionRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_task

> models::UserTaskResult get_user_task(user_task_key)
Get user task

Get the user task by the user task key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The user task key. | [required] |

### Return type

[**models::UserTaskResult**](UserTaskResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_task_form

> models::FormResult get_user_task_form(user_task_key)
Get user task form

Get the form of a user task. Note that this endpoint will only return linked forms. This endpoint does not support embedded forms. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The user task key. | [required] |

### Return type

[**models::FormResult**](FormResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_user_task_audit_logs

> models::AuditLogSearchQueryResult search_user_task_audit_logs(user_task_key, user_task_audit_log_search_query_request)
Search user task audit logs

Search for user task audit logs based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task. | [required] |
**user_task_audit_log_search_query_request** | Option<[**UserTaskAuditLogSearchQueryRequest**](UserTaskAuditLogSearchQueryRequest.md)> |  |  |

### Return type

[**models::AuditLogSearchQueryResult**](AuditLogSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_user_task_effective_variables

> models::VariableSearchQueryResult search_user_task_effective_variables(user_task_key, truncate_values, search_user_task_effective_variables_request)
Search user task effective variables

Search for the effective variables of a user task. This endpoint returns deduplicated variables where each variable name appears at most once. When the same variable name exists at multiple scope levels in the scope hierarchy, the value from the innermost scope (closest to the user task) takes precedence. This is useful for retrieving the actual runtime state of variables as seen by the user task. By default, long variable values in the response are truncated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task. | [required] |
**truncate_values** | Option<**bool**> | When true (default), long variable values in the response are truncated. When false, full variable values are returned. |  |
**search_user_task_effective_variables_request** | Option<[**SearchUserTaskEffectiveVariablesRequest**](SearchUserTaskEffectiveVariablesRequest.md)> |  |  |

### Return type

[**models::VariableSearchQueryResult**](VariableSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_user_task_variables

> models::VariableSearchQueryResult search_user_task_variables(user_task_key, truncate_values, user_task_variable_search_query_request)
Search user task variables

Search for user task variables based on given criteria. This endpoint returns all variable documents visible from the user task's scope, including variables from parent scopes in the scope hierarchy. If the same variable name exists at multiple scope levels, each scope's variable is returned as a separate result. Use the `/user-tasks/{userTaskKey}/effective-variables/search` endpoint to get deduplicated variables where the innermost scope takes precedence. By default, long variable values in the response are truncated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task. | [required] |
**truncate_values** | Option<**bool**> | When true (default), long variable values in the response are truncated. When false, full variable values are returned. |  |
**user_task_variable_search_query_request** | Option<[**UserTaskVariableSearchQueryRequest**](UserTaskVariableSearchQueryRequest.md)> |  |  |

### Return type

[**models::VariableSearchQueryResult**](VariableSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_user_tasks

> models::UserTaskSearchQueryResult search_user_tasks(user_task_search_query)
Search user tasks

Search for user tasks based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_search_query** | Option<[**UserTaskSearchQuery**](UserTaskSearchQuery.md)> |  |  |

### Return type

[**models::UserTaskSearchQueryResult**](UserTaskSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_user_task

> unassign_user_task(user_task_key)
Unassign user task

Removes the assignee of a task with the given key. Unassignment waits for blocking task listeners on this lifecycle transition. If listener processing is delayed beyond the request timeout, this endpoint can return 504. Other gateway timeout causes are also possible. Retry with backoff and inspect listener worker availability and logs when this repeats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_task

> update_user_task(user_task_key, user_task_update_request)
Update user task

Update a user task with the given key. Updates wait for blocking task listeners on this lifecycle transition. If listener processing is delayed beyond the request timeout, this endpoint can return 504. Other gateway timeout causes are also possible. Retry with backoff and inspect listener worker availability and logs when this repeats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_task_key** | **UserTaskKey** | The key of the user task to update. | [required] |
**user_task_update_request** | Option<[**UserTaskUpdateRequest**](UserTaskUpdateRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

