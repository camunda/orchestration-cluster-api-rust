# \RecoveryApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**change_cluster_mode**](RecoveryApi.md#change_cluster_mode) | **PATCH** /mode | Change cluster mode
[**change_cluster_mode_as_cluster_admin**](RecoveryApi.md#change_cluster_mode_as_cluster_admin) | **PATCH** /cluster/v2/mode | Change the cluster mode of one or every physical tenant
[**get_restore_status**](RecoveryApi.md#get_restore_status) | **GET** /restore | Get the status of the restore that is currently in progress
[**restore**](RecoveryApi.md#restore) | **POST** /restore | Restore from a backup
[**restore_as_cluster_admin**](RecoveryApi.md#restore_as_cluster_admin) | **POST** /cluster/v2/restore | Restore one or every physical tenant from a backup



## change_cluster_mode

> models::ClusterModeChangeResponse change_cluster_mode(mode, dry_run)
Change cluster mode

Transitions the cluster between processing and recovery mode. This is a non-blocking operation: the request is acknowledged once the change has been accepted, before the transition itself has completed. Entering recovery mode deactivates all partitions so that only a restricted set of read-only operations remains available; exiting recovery mode returns the cluster to normal processing. Returns the planned cluster change so its progress can be monitored via the topology.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mode** | [**Mode**](Mode.md) | The target cluster mode. | [required] |
**dry_run** | Option<**bool**> | If true, the requested change is only validated and the resulting plan is returned, without applying it to the cluster. |  |[default to false]

### Return type

[**models::ClusterModeChangeResponse**](ClusterModeChangeResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## change_cluster_mode_as_cluster_admin

> models::ClusterModeChangeResponse change_cluster_mode_as_cluster_admin(mode, physical_tenant_id, dry_run)
Change the cluster mode of one or every physical tenant

Transitions physical tenants between processing and recovery mode.  If the `physicalTenantId` parameter is not provided, all available physical tenants are transitioned individually.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mode** | [**Mode**](Mode.md) | The target cluster mode. | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |
**dry_run** | Option<**bool**> | If true, the requested change is only validated and the resulting plan is returned, without applying it to the cluster. |  |[default to false]

### Return type

[**models::ClusterModeChangeResponse**](ClusterModeChangeResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_restore_status

> models::RestoreStatusResponse get_restore_status()
Get the status of the restore that is currently in progress

Returns the status of the restore that is currently in progress, reported per broker and per partition. There is at most one restore in flight at any time. Once the restore has finished this endpoint returns 404; the per-partition detail is not retained after completion.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::RestoreStatusResponse**](RestoreStatusResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restore

> models::ClusterRestoreResponse restore(restore_request, dry_run)
Restore from a backup

Restores the cluster from a backup. The restore is described either by a single backup ID or by a time range (`from`/`to`) that selects the backups to restore. This endpoint is only accessible while the cluster is in recovery mode; requests are rejected otherwise. The request is validated and acknowledged, but the restore itself is performed asynchronously.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**restore_request** | [**RestoreRequest**](RestoreRequest.md) |  | [required] |
**dry_run** | Option<**bool**> | If true, the requested change is only validated and the resulting plan is returned, without applying it to the cluster. |  |[default to false]

### Return type

[**models::ClusterRestoreResponse**](ClusterRestoreResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restore_as_cluster_admin

> models::ClusterRestoreResponse restore_as_cluster_admin(cluster_restore_request, physical_tenant_id, dry_run)
Restore one or every physical tenant from a backup

Restores physical tenants from backups. The restore is described either by a list of backup IDs or by a time range (`from`/`to`) that selects the backups to restore. Restores are only accepted while the targeted physical tenants are in recovery mode; requests are rejected otherwise. The request is validated and acknowledged, but the restore itself is performed asynchronously.  If the `physicalTenantId` parameter is provided, only that physical tenant is restored and `overrides` must be omitted.  If it is not provided, every physical tenant of the cluster is restored: those named in `overrides` with their own backup selection, all others with the selection at the top level of the request body.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cluster_restore_request** | [**ClusterRestoreRequest**](ClusterRestoreRequest.md) |  | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |
**dry_run** | Option<**bool**> | If true, the requested change is only validated and the resulting plan is returned, without applying it to the cluster. |  |[default to false]

### Return type

[**models::ClusterRestoreResponse**](ClusterRestoreResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

