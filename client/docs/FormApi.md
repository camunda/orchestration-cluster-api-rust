# \FormApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_form_by_key**](FormApi.md#get_form_by_key) | **GET** /forms/{formKey} | Get form by key



## get_form_by_key

> models::FormResult get_form_by_key(form_key)
Get form by key

Get a form by its unique form key. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**form_key** | **FormKey** | The form key. | [required] |

### Return type

[**models::FormResult**](FormResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

