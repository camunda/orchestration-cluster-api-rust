# \RecoveryApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**change_cluster_mode**](RecoveryApi.md#change_cluster_mode) | **PATCH** /mode | Change cluster mode
[**restore**](RecoveryApi.md#restore) | **POST** /restore | Restore from a backup



## change_cluster_mode

> models::ClusterModeChangeResponse change_cluster_mode(mode, dry_run)
Change cluster mode

Transitions the cluster between processing and recovery mode. This is a non-blocking operation: the request is acknowledged once the change has been accepted, before the transition itself has completed. Entering recovery mode deactivates all partitions so that only a restricted set of read-only operations remains available; exiting recovery mode returns the cluster to normal processing. Returns the planned cluster change so its progress can be monitored via the topology.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mode** | **String** | The target cluster mode. | [required] |
**dry_run** | Option<**bool**> | If true, the requested change is only validated and the resulting plan is returned, without applying it to the cluster. |  |[default to false]

### Return type

[**models::ClusterModeChangeResponse**](ClusterModeChangeResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restore

> models::ClusterModeChangeResponse restore(restore_request)
Restore from a backup

Restores the cluster from a backup. The restore is described either by a single backup ID or by a time range (`from`/`to`) that selects the backups to restore. This endpoint is only accessible while the cluster is in recovery mode; requests are rejected otherwise. The request is validated and acknowledged, but the restore itself is performed asynchronously.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**restore_request** | [**RestoreRequest**](RestoreRequest.md) |  | [required] |

### Return type

[**models::ClusterModeChangeResponse**](ClusterModeChangeResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

