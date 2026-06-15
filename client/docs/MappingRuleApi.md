# \MappingRuleApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_mapping_rule**](MappingRuleApi.md#create_mapping_rule) | **POST** /mapping-rules | Create mapping rule
[**delete_mapping_rule**](MappingRuleApi.md#delete_mapping_rule) | **DELETE** /mapping-rules/{mappingRuleId} | Delete a mapping rule
[**get_mapping_rule**](MappingRuleApi.md#get_mapping_rule) | **GET** /mapping-rules/{mappingRuleId} | Get a mapping rule
[**search_mapping_rule**](MappingRuleApi.md#search_mapping_rule) | **POST** /mapping-rules/search | Search mapping rules
[**update_mapping_rule**](MappingRuleApi.md#update_mapping_rule) | **PUT** /mapping-rules/{mappingRuleId} | Update mapping rule



## create_mapping_rule

> models::MappingRuleCreateResult create_mapping_rule(mapping_rule_create_request)
Create mapping rule

Create a new mapping rule 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mapping_rule_create_request** | Option<[**MappingRuleCreateRequest**](MappingRuleCreateRequest.md)> |  |  |

### Return type

[**models::MappingRuleCreateResult**](MappingRuleCreateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_mapping_rule

> delete_mapping_rule(mapping_rule_id)
Delete a mapping rule

Deletes the mapping rule with the given ID. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mapping_rule_id** | **String** | The ID of the mapping rule to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_mapping_rule

> models::MappingRuleResult get_mapping_rule(mapping_rule_id)
Get a mapping rule

Gets the mapping rule with the given ID. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mapping_rule_id** | **String** | The ID of the mapping rule to get. | [required] |

### Return type

[**models::MappingRuleResult**](MappingRuleResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_mapping_rule

> models::MappingRuleSearchQueryResult search_mapping_rule(mapping_rule_search_query_request)
Search mapping rules

Search for mapping rules based on given criteria. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mapping_rule_search_query_request** | Option<[**MappingRuleSearchQueryRequest**](MappingRuleSearchQueryRequest.md)> |  |  |

### Return type

[**models::MappingRuleSearchQueryResult**](MappingRuleSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_mapping_rule

> models::MappingRuleUpdateResult update_mapping_rule(mapping_rule_id, mapping_rule_update_request)
Update mapping rule

Update a mapping rule. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mapping_rule_id** | **String** | The ID of the mapping rule to update. | [required] |
**mapping_rule_update_request** | Option<[**MappingRuleUpdateRequest**](MappingRuleUpdateRequest.md)> |  |  |

### Return type

[**models::MappingRuleUpdateResult**](MappingRuleUpdateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

