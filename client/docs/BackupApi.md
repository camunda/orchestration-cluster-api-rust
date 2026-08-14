# \BackupApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_history_backup**](BackupApi.md#delete_history_backup) | **DELETE** /backups/history/{backupId} | Delete history backup
[**delete_runtime_backup**](BackupApi.md#delete_runtime_backup) | **DELETE** /backups/runtime/{backupId} | Delete runtime backup
[**delete_runtime_backup_state**](BackupApi.md#delete_runtime_backup_state) | **DELETE** /backups/runtime/state | Delete runtime backup state
[**get_history_backup**](BackupApi.md#get_history_backup) | **GET** /backups/history/{backupId} | Get history backup
[**get_runtime_backup**](BackupApi.md#get_runtime_backup) | **GET** /backups/runtime/{backupId} | Get runtime backup
[**get_runtime_backup_state**](BackupApi.md#get_runtime_backup_state) | **GET** /backups/runtime/state | Get runtime backup state
[**list_history_backups**](BackupApi.md#list_history_backups) | **GET** /backups/history | List history backups
[**list_runtime_backups**](BackupApi.md#list_runtime_backups) | **GET** /backups/runtime | List runtime backups
[**sync_runtime_backup_state**](BackupApi.md#sync_runtime_backup_state) | **POST** /backups/runtime/state/sync | Force-write runtime backup state
[**take_history_backup**](BackupApi.md#take_history_backup) | **POST** /backups/history | Take a history backup
[**take_runtime_backup**](BackupApi.md#take_runtime_backup) | **POST** /backups/runtime | Take a runtime backup



## delete_history_backup

> delete_history_backup(backup_id)
Delete history backup

Deletes the history backup with the given id, by deleting every snapshot that makes it up.  Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_runtime_backup

> delete_runtime_backup(backup_id)
Delete runtime backup

Deletes the runtime backup with the given id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_runtime_backup_state

> delete_runtime_backup_state()
Delete runtime backup state

Resets the runtime backup state of every partition of the physical tenant, clearing all checkpoint info, backup info, checkpoint metadata, and backup ranges. Used when switching backup stores. 

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


## get_history_backup

> models::HistoryBackupInfo get_history_backup(backup_id)
Get history backup

Returns detailed status of the history backup with the given id.  Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |

### Return type

[**models::HistoryBackupInfo**](HistoryBackupInfo.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_runtime_backup

> models::BackupInfo get_runtime_backup(backup_id)
Get runtime backup

Returns detailed status of the runtime backup with the given id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |

### Return type

[**models::BackupInfo**](BackupInfo.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_runtime_backup_state

> models::RuntimeBackupState get_runtime_backup_state()
Get runtime backup state

Returns the current checkpoint and backup state of every partition of the physical tenant. Unlike the `backupRuntime` actuator, this fails the whole request if the checkpoint state or the backup ranges cannot be retrieved from any partition, instead of silently returning an empty section. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::RuntimeBackupState**](RuntimeBackupState.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_history_backups

> Vec<models::HistoryBackupInfo> list_history_backups(prefix, verbose)
List history backups

Returns a list of all available history backups of the physical tenant, with their state and additional info, most recent first by snapshot start time.  Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**prefix** | Option<**String**> | A prefix that backup ids must match, ending in a single '*'. If omitted, all backups are returned.  |  |
**verbose** | Option<**bool**> | Whether to ask the secondary storage for snapshot-level detail. Setting this to `false` makes the query cheaper, but the store then reports neither snapshot state nor start time, so both the per-snapshot `details` and the aggregated `state` are incomplete and the listing order is unspecified.  |  |[default to true]

### Return type

[**Vec<models::HistoryBackupInfo>**](HistoryBackupInfo.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_runtime_backups

> Vec<models::BackupInfo> list_runtime_backups(prefix)
List runtime backups

Returns a list of all available runtime backups of the physical tenant, with their state and additional info, sorted in descending order of backupId. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**prefix** | Option<**String**> | A prefix that backup ids must match, ending in a single '*'. If omitted, all backups are returned.  |  |

### Return type

[**Vec<models::BackupInfo>**](BackupInfo.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_runtime_backup_state

> models::RuntimeBackupState sync_runtime_backup_state()
Force-write runtime backup state

Force-writes the checkpoint and backup metadata of every partition of the physical tenant to the backup store, independent of any backup being taken or confirmed, and returns the updated state. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::RuntimeBackupState**](RuntimeBackupState.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_history_backup

> models::TakeHistoryBackupResponse take_history_backup(take_history_backup_request)
Take a history backup

Triggers a backup of the physical tenant's history, by scheduling a snapshot of every secondary storage index it owns.  Unlike runtime backups, history backups have no generated-id mode: `backupId` is always required.  Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**take_history_backup_request** | [**TakeHistoryBackupRequest**](TakeHistoryBackupRequest.md) |  | [required] |

### Return type

[**models::TakeHistoryBackupResponse**](TakeHistoryBackupResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_runtime_backup

> models::TakeRuntimeBackupResponse take_runtime_backup(take_runtime_backup_request)
Take a runtime backup

Triggers a backup of runtime data on all partitions of the physical tenant.  The `backupId` must be omitted if continuous backups and/or a backup or checkpoint schedule is enabled for the physical tenant, as the id is generated automatically. Otherwise, `backupId` is required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**take_runtime_backup_request** | Option<[**TakeRuntimeBackupRequest**](TakeRuntimeBackupRequest.md)> |  |  |

### Return type

[**models::TakeRuntimeBackupResponse**](TakeRuntimeBackupResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

