# \ElementInstanceApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_element_instance_variables**](ElementInstanceApi.md#create_element_instance_variables) | **PUT** /element-instances/{elementInstanceKey}/variables | Update element instance variables
[**get_element_instance**](ElementInstanceApi.md#get_element_instance) | **GET** /element-instances/{elementInstanceKey} | Get element instance
[**search_element_instance_incidents**](ElementInstanceApi.md#search_element_instance_incidents) | **POST** /element-instances/{elementInstanceKey}/incidents/search | Search for incidents of a specific element instance
[**search_element_instance_wait_states**](ElementInstanceApi.md#search_element_instance_wait_states) | **POST** /element-instances/wait-states/search | Search element instance wait states
[**search_element_instances**](ElementInstanceApi.md#search_element_instances) | **POST** /element-instances/search | Search element instances



## create_element_instance_variables

> create_element_instance_variables(element_instance_key, set_variable_request)
Update element instance variables

Updates all the variables of a particular scope (for example, process instance, element instance) with the given variable data. Specify the element instance in the `elementInstanceKey` parameter. Variable updates can be delayed by listener-related processing; if processing exceeds the request timeout, this endpoint can return 504. Other gateway timeout causes are also possible. Retry with backoff and inspect listener worker availability and logs when this repeats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**element_instance_key** | **ElementInstanceKey** | The key of the element instance to update the variables for. This can be the process instance key (as obtained during instance creation), or a given element, such as a service task (see the `elementInstanceKey` on the job message).  | [required] |
**set_variable_request** | [**SetVariableRequest**](SetVariableRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_element_instance

> models::ElementInstanceResult get_element_instance(element_instance_key)
Get element instance

Returns element instance as JSON.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**element_instance_key** | **ElementInstanceKey** | The assigned key of the element instance, which acts as a unique identifier for this element instance. | [required] |

### Return type

[**models::ElementInstanceResult**](ElementInstanceResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_element_instance_incidents

> models::IncidentSearchQueryResult search_element_instance_incidents(element_instance_key, incident_search_query)
Search for incidents of a specific element instance

Search for incidents caused by the specified element instance, including incidents of any child instances created from this element instance.  Although the `elementInstanceKey` is provided as a path parameter to indicate the root element instance, you may also include an `elementInstanceKey` within the filter object to narrow results to specific child element instances. This is useful, for example, if you want to isolate incidents associated with nested or subordinate elements within the given element instance while excluding incidents directly tied to the root element itself. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**element_instance_key** | **ElementInstanceKey** | The unique key of the element instance to search incidents for. | [required] |
**incident_search_query** | [**IncidentSearchQuery**](IncidentSearchQuery.md) |  | [required] |

### Return type

[**models::IncidentSearchQueryResult**](IncidentSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_element_instance_wait_states

> models::ElementInstanceWaitStateQueryResult search_element_instance_wait_states(element_instance_wait_state_query)
Search element instance wait states

Returns the wait states for element instances matching the given filter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**element_instance_wait_state_query** | Option<[**ElementInstanceWaitStateQuery**](ElementInstanceWaitStateQuery.md)> |  |  |

### Return type

[**models::ElementInstanceWaitStateQueryResult**](ElementInstanceWaitStateQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_element_instances

> models::ElementInstanceSearchQueryResult search_element_instances(element_instance_search_query)
Search element instances

Search for element instances based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**element_instance_search_query** | Option<[**ElementInstanceSearchQuery**](ElementInstanceSearchQuery.md)> |  |  |

### Return type

[**models::ElementInstanceSearchQueryResult**](ElementInstanceSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

