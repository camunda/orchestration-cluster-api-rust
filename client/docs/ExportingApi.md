# \ExportingApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_exporting_status**](ExportingApi.md#get_exporting_status) | **GET** /exporting | Get exporting status
[**pause_exporting**](ExportingApi.md#pause_exporting) | **POST** /exporting/pause | Pause exporting
[**resume_exporting**](ExportingApi.md#resume_exporting) | **POST** /exporting/resume | Resume exporting



## get_exporting_status

> models::ExportingStatusResponse get_exporting_status()
Get exporting status

Returns the exporting status of the physical tenant, aggregated over every replica of every one of its partitions.  Because pause and resume are applied to all replicas, the status is only a single phase if every replica reports that phase; otherwise it is `MIXED`, which means a pause or resume is still in flight or was only partially applied. Backup tooling should treat only `PAUSED` and `SOFT_PAUSED` as confirmation that exporting is paused. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ExportingStatusResponse**](ExportingStatusResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pause_exporting

> pause_exporting(soft)
Pause exporting

Pauses exporting on all partitions of the physical tenant. While paused, exported records are not committed, so the log is not compacted for the affected partitions.  With `soft=true`, exporting continues to run but its position is not committed, so the state after resuming is identical to a hard pause; use this variant when exporting must keep progressing (e.g. to avoid falling behind) while still preventing log compaction, such as during a backup. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**soft** | Option<**bool**> | If true, soft-pauses exporting instead of a hard pause. |  |[default to false]

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resume_exporting

> resume_exporting()
Resume exporting

Resumes exporting on all partitions of the physical tenant after a pause or soft pause. 

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

