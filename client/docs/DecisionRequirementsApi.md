# \DecisionRequirementsApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_decision_requirements**](DecisionRequirementsApi.md#get_decision_requirements) | **GET** /decision-requirements/{decisionRequirementsKey} | Get decision requirements
[**get_decision_requirements_xml**](DecisionRequirementsApi.md#get_decision_requirements_xml) | **GET** /decision-requirements/{decisionRequirementsKey}/xml | Get decision requirements XML
[**search_decision_requirements**](DecisionRequirementsApi.md#search_decision_requirements) | **POST** /decision-requirements/search | Search decision requirements



## get_decision_requirements

> models::DecisionRequirementsResult get_decision_requirements(decision_requirements_key)
Get decision requirements

Returns Decision Requirements as JSON.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_requirements_key** | **DecisionRequirementsKey** | The assigned key of the decision requirements, which acts as a unique identifier for this decision requirements. | [required] |

### Return type

[**models::DecisionRequirementsResult**](DecisionRequirementsResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_decision_requirements_xml

> String get_decision_requirements_xml(decision_requirements_key)
Get decision requirements XML

Returns decision requirements as XML.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_requirements_key** | **DecisionRequirementsKey** | The assigned key of the decision requirements, which acts as a unique identifier for this decision. | [required] |

### Return type

**String**

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/xml, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_decision_requirements

> models::DecisionRequirementsSearchQueryResult search_decision_requirements(decision_requirements_search_query)
Search decision requirements

Search for decision requirements based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_requirements_search_query** | Option<[**DecisionRequirementsSearchQuery**](DecisionRequirementsSearchQuery.md)> |  |  |

### Return type

[**models::DecisionRequirementsSearchQueryResult**](DecisionRequirementsSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

