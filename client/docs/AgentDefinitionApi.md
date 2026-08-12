# \AgentDefinitionApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_agent_definition**](AgentDefinitionApi.md#get_agent_definition) | **GET** /agent-definitions/{agentDefinitionKey} | Get agent definition
[**search_agent_definitions**](AgentDefinitionApi.md#search_agent_definitions) | **POST** /agent-definitions/search | Search agent definitions



## get_agent_definition

> models::AgentDefinitionResult get_agent_definition(agent_definition_key)
Get agent definition

Returns an agent definition by key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_definition_key** | **AgentDefinitionKey** | The assigned key of the agent definition, which acts as a unique identifier for this agent definition. | [required] |

### Return type

[**models::AgentDefinitionResult**](AgentDefinitionResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_agent_definitions

> models::AgentDefinitionSearchQueryResult search_agent_definitions(agent_definition_search_query)
Search agent definitions

Search for agent definitions based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**agent_definition_search_query** | Option<[**AgentDefinitionSearchQuery**](AgentDefinitionSearchQuery.md)> |  |  |

### Return type

[**models::AgentDefinitionSearchQueryResult**](AgentDefinitionSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

