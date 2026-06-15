# \IncidentApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_incident**](IncidentApi.md#get_incident) | **GET** /incidents/{incidentKey} | Get incident
[**get_process_instance_statistics_by_definition**](IncidentApi.md#get_process_instance_statistics_by_definition) | **POST** /incidents/statistics/process-instances-by-definition | Get process instance statistics by definition
[**get_process_instance_statistics_by_error**](IncidentApi.md#get_process_instance_statistics_by_error) | **POST** /incidents/statistics/process-instances-by-error | Get process instance statistics by error
[**resolve_incident**](IncidentApi.md#resolve_incident) | **POST** /incidents/{incidentKey}/resolution | Resolve incident
[**search_incidents**](IncidentApi.md#search_incidents) | **POST** /incidents/search | Search incidents



## get_incident

> models::IncidentResult get_incident(incident_key)
Get incident

Returns incident as JSON. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**incident_key** | **IncidentKey** | The assigned key of the incident, which acts as a unique identifier for this incident. | [required] |

### Return type

[**models::IncidentResult**](IncidentResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_instance_statistics_by_definition

> models::IncidentProcessInstanceStatisticsByDefinitionQueryResult get_process_instance_statistics_by_definition(incident_process_instance_statistics_by_definition_query)
Get process instance statistics by definition

Returns statistics for active process instances with incidents, grouped by process definition. The result set is scoped to a specific incident error hash code, which must be provided as a filter in the request body. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**incident_process_instance_statistics_by_definition_query** | [**IncidentProcessInstanceStatisticsByDefinitionQuery**](IncidentProcessInstanceStatisticsByDefinitionQuery.md) |  | [required] |

### Return type

[**models::IncidentProcessInstanceStatisticsByDefinitionQueryResult**](IncidentProcessInstanceStatisticsByDefinitionQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_instance_statistics_by_error

> models::IncidentProcessInstanceStatisticsByErrorQueryResult get_process_instance_statistics_by_error(incident_process_instance_statistics_by_error_query)
Get process instance statistics by error

Returns statistics for active process instances that currently have active incidents, grouped by incident error hash code. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**incident_process_instance_statistics_by_error_query** | Option<[**IncidentProcessInstanceStatisticsByErrorQuery**](IncidentProcessInstanceStatisticsByErrorQuery.md)> |  |  |

### Return type

[**models::IncidentProcessInstanceStatisticsByErrorQueryResult**](IncidentProcessInstanceStatisticsByErrorQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resolve_incident

> resolve_incident(incident_key, incident_resolution_request)
Resolve incident

Marks the incident as resolved; most likely a call to Update job will be necessary to reset the job's retries, followed by this call. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**incident_key** | **IncidentKey** | Key of the incident to resolve. | [required] |
**incident_resolution_request** | Option<[**IncidentResolutionRequest**](IncidentResolutionRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_incidents

> models::IncidentSearchQueryResult search_incidents(incident_search_query)
Search incidents

Search for incidents based on given criteria. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**incident_search_query** | Option<[**IncidentSearchQuery**](IncidentSearchQuery.md)> |  |  |

### Return type

[**models::IncidentSearchQueryResult**](IncidentSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

