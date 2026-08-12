# \AuthenticationApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_authentication**](AuthenticationApi.md#get_authentication) | **GET** /authentication/me | Get current user
[**search_own_authorizations**](AuthenticationApi.md#search_own_authorizations) | **POST** /authentication/me/authorizations/search | Search own authorizations



## get_authentication

> models::CamundaUserResult get_authentication()
Get current user

Retrieves the current authenticated user.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::CamundaUserResult**](CamundaUserResult.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_own_authorizations

> models::AuthorizationSearchResult search_own_authorizations(authorization_search_query)
Search own authorizations

Search for the current authenticated principal's own authorization records — including authorizations granted directly to the user or client, as well as those granted via a group, role, or mapping rule the principal belongs to.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization_search_query** | Option<[**AuthorizationSearchQuery**](AuthorizationSearchQuery.md)> |  |  |

### Return type

[**models::AuthorizationSearchResult**](AuthorizationSearchResult.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

