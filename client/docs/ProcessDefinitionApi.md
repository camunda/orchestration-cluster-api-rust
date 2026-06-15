# \ProcessDefinitionApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_process_definition**](ProcessDefinitionApi.md#get_process_definition) | **GET** /process-definitions/{processDefinitionKey} | Get process definition
[**get_process_definition_instance_statistics**](ProcessDefinitionApi.md#get_process_definition_instance_statistics) | **POST** /process-definitions/statistics/process-instances | Get process instance statistics
[**get_process_definition_instance_version_statistics**](ProcessDefinitionApi.md#get_process_definition_instance_version_statistics) | **POST** /process-definitions/statistics/process-instances-by-version | Get process instance statistics by version
[**get_process_definition_message_subscription_statistics**](ProcessDefinitionApi.md#get_process_definition_message_subscription_statistics) | **POST** /process-definitions/statistics/message-subscriptions | Get message subscription statistics
[**get_process_definition_statistics**](ProcessDefinitionApi.md#get_process_definition_statistics) | **POST** /process-definitions/{processDefinitionKey}/statistics/element-instances | Get process definition statistics
[**get_process_definition_xml**](ProcessDefinitionApi.md#get_process_definition_xml) | **GET** /process-definitions/{processDefinitionKey}/xml | Get process definition XML
[**get_start_process_form**](ProcessDefinitionApi.md#get_start_process_form) | **GET** /process-definitions/{processDefinitionKey}/form | Get process start form
[**search_process_definitions**](ProcessDefinitionApi.md#search_process_definitions) | **POST** /process-definitions/search | Search process definitions



## get_process_definition

> models::ProcessDefinitionResult get_process_definition(process_definition_key)
Get process definition

Returns process definition as JSON.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_key** | **ProcessDefinitionKey** | The assigned key of the process definition, which acts as a unique identifier for this process definition.  | [required] |

### Return type

[**models::ProcessDefinitionResult**](ProcessDefinitionResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_definition_instance_statistics

> models::ProcessDefinitionInstanceStatisticsQueryResult get_process_definition_instance_statistics(process_definition_instance_statistics_query)
Get process instance statistics

Get statistics about process instances, grouped by process definition and tenant. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_instance_statistics_query** | Option<[**ProcessDefinitionInstanceStatisticsQuery**](ProcessDefinitionInstanceStatisticsQuery.md)> |  |  |

### Return type

[**models::ProcessDefinitionInstanceStatisticsQueryResult**](ProcessDefinitionInstanceStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_definition_instance_version_statistics

> models::ProcessDefinitionInstanceVersionStatisticsQueryResult get_process_definition_instance_version_statistics(process_definition_instance_version_statistics_query)
Get process instance statistics by version

Get statistics about process instances, grouped by version for a given process definition. The process definition ID must be provided as a required field in the request body filter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_instance_version_statistics_query** | [**ProcessDefinitionInstanceVersionStatisticsQuery**](ProcessDefinitionInstanceVersionStatisticsQuery.md) |  | [required] |

### Return type

[**models::ProcessDefinitionInstanceVersionStatisticsQueryResult**](ProcessDefinitionInstanceVersionStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_definition_message_subscription_statistics

> models::ProcessDefinitionMessageSubscriptionStatisticsQueryResult get_process_definition_message_subscription_statistics(process_definition_message_subscription_statistics_query)
Get message subscription statistics

Get message subscription statistics, grouped by process definition. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_message_subscription_statistics_query** | Option<[**ProcessDefinitionMessageSubscriptionStatisticsQuery**](ProcessDefinitionMessageSubscriptionStatisticsQuery.md)> |  |  |

### Return type

[**models::ProcessDefinitionMessageSubscriptionStatisticsQueryResult**](ProcessDefinitionMessageSubscriptionStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_definition_statistics

> models::ProcessDefinitionElementStatisticsQueryResult get_process_definition_statistics(process_definition_key, process_definition_element_statistics_query)
Get process definition statistics

Get statistics about elements in currently running process instances by process definition key and search filter.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_key** | **ProcessDefinitionKey** | The assigned key of the process definition, which acts as a unique identifier for this process definition. | [required] |
**process_definition_element_statistics_query** | Option<[**ProcessDefinitionElementStatisticsQuery**](ProcessDefinitionElementStatisticsQuery.md)> |  |  |

### Return type

[**models::ProcessDefinitionElementStatisticsQueryResult**](ProcessDefinitionElementStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_definition_xml

> String get_process_definition_xml(process_definition_key)
Get process definition XML

Returns process definition as XML.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_key** | **ProcessDefinitionKey** | The assigned key of the process definition, which acts as a unique identifier for this process definition.  | [required] |

### Return type

**String**

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/xml, text/plain, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_start_process_form

> models::FormResult get_start_process_form(process_definition_key)
Get process start form

Get the start form of a process. Note that this endpoint will only return linked forms. This endpoint does not support embedded forms. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_key** | **ProcessDefinitionKey** | The process key. | [required] |

### Return type

[**models::FormResult**](FormResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_process_definitions

> models::ProcessDefinitionSearchQueryResult search_process_definitions(process_definition_search_query)
Search process definitions

Search for process definitions based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_definition_search_query** | Option<[**ProcessDefinitionSearchQuery**](ProcessDefinitionSearchQuery.md)> |  |  |

### Return type

[**models::ProcessDefinitionSearchQueryResult**](ProcessDefinitionSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

