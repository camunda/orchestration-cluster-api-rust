# \AgentInstanceApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_agent_instance**](AgentInstanceApi.md#create_agent_instance) | **POST** /agent-instances | Create agent instance
[**create_agent_instance_history_item**](AgentInstanceApi.md#create_agent_instance_history_item) | **POST** /agent-instances/{agentInstanceKey}/history | Create agent instance history item
[**get_agent_instance**](AgentInstanceApi.md#get_agent_instance) | **GET** /agent-instances/{agentInstanceKey} | Get agent instance
[**search_agent_instance_history**](AgentInstanceApi.md#search_agent_instance_history) | **POST** /agent-instances/{agentInstanceKey}/history/search | Search agent instance history
[**search_agent_instances**](AgentInstanceApi.md#search_agent_instances) | **POST** /agent-instances/search | Search agent instances
[**update_agent_instance**](AgentInstanceApi.md#update_agent_instance) | **PATCH** /agent-instances/{agentInstanceKey} | Update agent instance



## create_agent_instance

> models::AgentInstanceCreationResult create_agent_instance(agent_instance_creation_request)
Create agent instance

Creates a new agent instance. The returned key identifies the instance and must be used in subsequent update and query calls. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_instance_creation_request** | [**AgentInstanceCreationRequest**](AgentInstanceCreationRequest.md) |  | [required] |

### Return type

[**models::AgentInstanceCreationResult**](AgentInstanceCreationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_agent_instance_history_item

> models::AgentInstanceHistoryItemCreationResult create_agent_instance_history_item(agent_instance_key, agent_instance_history_item_request)
Create agent instance history item

Appends a single history item to an agent instance's conversation history. The created item has commitStatus PENDING until the job identified by jobLease completes successfully, at which point it transitions to COMMITTED. If the job fails or is superseded by a retry, the item is marked DISCARDED. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_instance_key** | **AgentInstanceKey** | The key of the agent instance to append the history item to. | [required] |
**agent_instance_history_item_request** | [**AgentInstanceHistoryItemRequest**](AgentInstanceHistoryItemRequest.md) |  | [required] |

### Return type

[**models::AgentInstanceHistoryItemCreationResult**](AgentInstanceHistoryItemCreationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_agent_instance

> models::AgentInstanceResult get_agent_instance(agent_instance_key)
Get agent instance

Returns agent instance as JSON.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_instance_key** | **AgentInstanceKey** | The key of the agent instance to retrieve. | [required] |

### Return type

[**models::AgentInstanceResult**](AgentInstanceResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_agent_instance_history

> models::AgentInstanceHistorySearchQueryResult search_agent_instance_history(agent_instance_key, agent_instance_history_search_query)
Search agent instance history

Searches the conversation history of an agent instance. Committed items are returned by default. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_instance_key** | **AgentInstanceKey** | The key of the agent instance whose history to search. | [required] |
**agent_instance_history_search_query** | Option<[**AgentInstanceHistorySearchQuery**](AgentInstanceHistorySearchQuery.md)> |  |  |

### Return type

[**models::AgentInstanceHistorySearchQueryResult**](AgentInstanceHistorySearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_agent_instances

> models::AgentInstanceSearchQueryResult search_agent_instances(agent_instance_search_query)
Search agent instances

Search for agent instances based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_instance_search_query** | Option<[**AgentInstanceSearchQuery**](AgentInstanceSearchQuery.md)> |  |  |

### Return type

[**models::AgentInstanceSearchQueryResult**](AgentInstanceSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_agent_instance

> update_agent_instance(agent_instance_key, agent_instance_update_request)
Update agent instance

Updates the mutable fields of an agent instance: status, metric counters, and tools. Metric values are treated as deltas and applied immediately to the aggregate counters. Tool updates replace the existing tool list. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_instance_key** | **AgentInstanceKey** | The key of the agent instance to update. | [required] |
**agent_instance_update_request** | [**AgentInstanceUpdateRequest**](AgentInstanceUpdateRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

