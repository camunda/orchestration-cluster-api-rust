# \SystemApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_system_configuration**](SystemApi.md#get_system_configuration) | **GET** /system/configuration | System configuration (alpha)
[**get_usage_metrics**](SystemApi.md#get_usage_metrics) | **GET** /system/usage-metrics | Get usage metrics



## get_system_configuration

> models::SystemConfigurationResponse get_system_configuration()
System configuration (alpha)

Returns the current system configuration. The response is an envelope that groups settings by feature area.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SystemConfigurationResponse**](SystemConfigurationResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_usage_metrics

> models::UsageMetricsResponse get_usage_metrics(start_time, end_time, tenant_id, with_tenants)
Get usage metrics

Retrieve the usage metrics based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**start_time** | **chrono::DateTime<chrono::FixedOffset>** | The start date for usage metrics, including this date. Value in ISO 8601 format. | [required] |
**end_time** | **chrono::DateTime<chrono::FixedOffset>** | The end date for usage metrics, including this date. Value in ISO 8601 format. | [required] |
**tenant_id** | Option<**String**> | Restrict results to a specific tenant ID. If not provided, results for all tenants are returned. |  |
**with_tenants** | Option<**bool**> | Whether to return tenant metrics in addition to the total metrics or not. Default false. |  |[default to false]

### Return type

[**models::UsageMetricsResponse**](UsageMetricsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

