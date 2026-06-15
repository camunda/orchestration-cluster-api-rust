# \AuthenticationApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_authentication**](AuthenticationApi.md#get_authentication) | **GET** /authentication/me | Get current user



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

