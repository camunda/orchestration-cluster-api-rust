# \GlobalListenerApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_global_task_listener**](GlobalListenerApi.md#create_global_task_listener) | **POST** /global-task-listeners | Create global user task listener
[**delete_global_task_listener**](GlobalListenerApi.md#delete_global_task_listener) | **DELETE** /global-task-listeners/{id} | Delete global user task listener
[**get_global_task_listener**](GlobalListenerApi.md#get_global_task_listener) | **GET** /global-task-listeners/{id} | Get global user task listener
[**search_global_task_listeners**](GlobalListenerApi.md#search_global_task_listeners) | **POST** /global-task-listeners/search | Search global user task listeners
[**update_global_task_listener**](GlobalListenerApi.md#update_global_task_listener) | **PUT** /global-task-listeners/{id} | Update global user task listener



## create_global_task_listener

> models::GlobalTaskListenerResult create_global_task_listener(create_global_task_listener_request)
Create global user task listener

Create a new global user task listener.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_global_task_listener_request** | [**CreateGlobalTaskListenerRequest**](CreateGlobalTaskListenerRequest.md) |  | [required] |

### Return type

[**models::GlobalTaskListenerResult**](GlobalTaskListenerResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_global_task_listener

> delete_global_task_listener(id)
Delete global user task listener

Deletes a global user task listener.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The id of the global user task listener to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_global_task_listener

> models::GlobalTaskListenerResult get_global_task_listener(id)
Get global user task listener

Get a global user task listener by its id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The id of the global user task listener. | [required] |

### Return type

[**models::GlobalTaskListenerResult**](GlobalTaskListenerResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_global_task_listeners

> models::GlobalTaskListenerSearchQueryResult search_global_task_listeners(global_task_listener_search_query_request)
Search global user task listeners

Search for global user task listeners based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**global_task_listener_search_query_request** | Option<[**GlobalTaskListenerSearchQueryRequest**](GlobalTaskListenerSearchQueryRequest.md)> |  |  |

### Return type

[**models::GlobalTaskListenerSearchQueryResult**](GlobalTaskListenerSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_global_task_listener

> models::GlobalTaskListenerResult update_global_task_listener(id, update_global_task_listener_request)
Update global user task listener

Updates a global user task listener.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The id of the global user task listener to update. | [required] |
**update_global_task_listener_request** | [**UpdateGlobalTaskListenerRequest**](UpdateGlobalTaskListenerRequest.md) |  | [required] |

### Return type

[**models::GlobalTaskListenerResult**](GlobalTaskListenerResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

