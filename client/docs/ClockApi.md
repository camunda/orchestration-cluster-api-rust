# \ClockApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**pin_clock**](ClockApi.md#pin_clock) | **PUT** /clock | Pin internal clock (alpha)
[**reset_clock**](ClockApi.md#reset_clock) | **POST** /clock/reset | Reset internal clock (alpha)



## pin_clock

> pin_clock(clock_pin_request)
Pin internal clock (alpha)

Set a precise, static time for the Zeebe engine's internal clock. When the clock is pinned, it remains at the specified time and does not advance. To change the time, the clock must be pinned again with a new timestamp.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**clock_pin_request** | [**ClockPinRequest**](ClockPinRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_clock

> reset_clock()
Reset internal clock (alpha)

Resets the Zeebe engine's internal clock to the current system time, enabling it to tick in real-time. This operation is useful for returning the clock to normal behavior after it has been pinned to a specific time.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

