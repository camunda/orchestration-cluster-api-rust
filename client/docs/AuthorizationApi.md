# \AuthorizationApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_authorization**](AuthorizationApi.md#create_authorization) | **POST** /authorizations | Create authorization
[**delete_authorization**](AuthorizationApi.md#delete_authorization) | **DELETE** /authorizations/{authorizationKey} | Delete authorization
[**get_authorization**](AuthorizationApi.md#get_authorization) | **GET** /authorizations/{authorizationKey} | Get authorization
[**search_authorizations**](AuthorizationApi.md#search_authorizations) | **POST** /authorizations/search | Search authorizations
[**update_authorization**](AuthorizationApi.md#update_authorization) | **PUT** /authorizations/{authorizationKey} | Update authorization



## create_authorization

> models::AuthorizationCreateResult create_authorization(authorization_request)
Create authorization

Create the authorization.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization_request** | [**AuthorizationRequest**](AuthorizationRequest.md) |  | [required] |

### Return type

[**models::AuthorizationCreateResult**](AuthorizationCreateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_authorization

> delete_authorization(authorization_key)
Delete authorization

Deletes the authorization with the given key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization_key** | **AuthorizationKey** | The key of the authorization to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_authorization

> models::AuthorizationResult get_authorization(authorization_key)
Get authorization

Get authorization by the given key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization_key** | **AuthorizationKey** | The key of the authorization to get. | [required] |

### Return type

[**models::AuthorizationResult**](AuthorizationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_authorizations

> models::AuthorizationSearchResult search_authorizations(authorization_search_query)
Search authorizations

Search for authorizations based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization_search_query** | Option<[**AuthorizationSearchQuery**](AuthorizationSearchQuery.md)> |  |  |

### Return type

[**models::AuthorizationSearchResult**](AuthorizationSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_authorization

> update_authorization(authorization_key, authorization_request)
Update authorization

Update the authorization with the given key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization_key** | **AuthorizationKey** | The key of the authorization to delete. | [required] |
**authorization_request** | [**AuthorizationRequest**](AuthorizationRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

