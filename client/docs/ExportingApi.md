# \ExportingApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_cluster_exporting_status**](ExportingApi.md#get_cluster_exporting_status) | **GET** /cluster/v2/exporting | Get exporting status of the whole cluster
[**get_exporting_status**](ExportingApi.md#get_exporting_status) | **GET** /exporting | Get exporting status
[**pause_cluster_exporting**](ExportingApi.md#pause_cluster_exporting) | **POST** /cluster/v2/exporting/pause | Pause exporting across the whole cluster
[**pause_exporting**](ExportingApi.md#pause_exporting) | **POST** /exporting/pause | Pause exporting
[**resume_cluster_exporting**](ExportingApi.md#resume_cluster_exporting) | **POST** /cluster/v2/exporting/resume | Resume exporting across the whole cluster
[**resume_exporting**](ExportingApi.md#resume_exporting) | **POST** /exporting/resume | Resume exporting



## get_cluster_exporting_status

> models::ExportingStatusResponse get_cluster_exporting_status()
Get exporting status of the whole cluster

Returns the exporting status of the whole cluster, folded over the exporting status of every physical tenant. Only `PAUSED` and `SOFT_PAUSED` confirm that exporting is paused cluster-wide; every other value means at least one physical tenant is not paused, so callers should keep polling. A physical tenant that itself reports `MIXED` makes the whole cluster `MIXED`.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

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


## pause_cluster_exporting

> pause_cluster_exporting(soft)
Pause exporting across the whole cluster

Pauses exporting on every physical tenant of the cluster in one call. With `soft=true`, every physical tenant is soft-paused instead.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

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


## resume_cluster_exporting

> resume_cluster_exporting()
Resume exporting across the whole cluster

Resumes exporting on every physical tenant of the cluster in one call, after a pause or soft pause.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

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

