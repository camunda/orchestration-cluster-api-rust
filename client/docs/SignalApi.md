# \SignalApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**broadcast_signal**](SignalApi.md#broadcast_signal) | **POST** /signals/broadcast | Broadcast signal



## broadcast_signal

> models::SignalBroadcastResult broadcast_signal(signal_broadcast_request)
Broadcast signal

Broadcasts a signal.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**signal_broadcast_request** | [**SignalBroadcastRequest**](SignalBroadcastRequest.md) |  | [required] |

### Return type

[**models::SignalBroadcastResult**](SignalBroadcastResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

