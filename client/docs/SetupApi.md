# \SetupApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_admin_user**](SetupApi.md#create_admin_user) | **POST** /setup/user | Create admin user



## create_admin_user

> models::UserCreateResult create_admin_user(user_request)
Create admin user

Creates a new user and assigns the admin role to it. This endpoint is only usable when users are managed in the Orchestration Cluster and while no user is assigned to the admin role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_request** | [**UserRequest**](UserRequest.md) |  | [required] |

### Return type

[**models::UserCreateResult**](UserCreateResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

