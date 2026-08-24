# \BackupApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_history_backup**](BackupApi.md#delete_history_backup) | **DELETE** /backups/history/{backupId} | Delete history backup
[**delete_history_backup_as_cluster_admin**](BackupApi.md#delete_history_backup_as_cluster_admin) | **DELETE** /cluster/v2/backups/history/{backupId} | Delete a history backup across physical tenants
[**delete_runtime_backup**](BackupApi.md#delete_runtime_backup) | **DELETE** /backups/runtime/{backupId} | Delete runtime backup
[**delete_runtime_backup_as_cluster_admin**](BackupApi.md#delete_runtime_backup_as_cluster_admin) | **DELETE** /cluster/v2/backups/runtime/{backupId} | Delete a runtime backup across physical tenants
[**delete_runtime_backup_state**](BackupApi.md#delete_runtime_backup_state) | **DELETE** /backups/runtime/state | Delete runtime backup state
[**delete_runtime_backup_state_as_cluster_admin**](BackupApi.md#delete_runtime_backup_state_as_cluster_admin) | **DELETE** /cluster/v2/backups/runtime/state | Delete runtime backup state across physical tenants
[**get_history_backup**](BackupApi.md#get_history_backup) | **GET** /backups/history/{backupId} | Get history backup
[**get_history_backup_as_cluster_admin**](BackupApi.md#get_history_backup_as_cluster_admin) | **GET** /cluster/v2/backups/history/{backupId} | Get a history backup across physical tenants
[**get_runtime_backup**](BackupApi.md#get_runtime_backup) | **GET** /backups/runtime/{backupId} | Get runtime backup
[**get_runtime_backup_as_cluster_admin**](BackupApi.md#get_runtime_backup_as_cluster_admin) | **GET** /cluster/v2/backups/runtime/{backupId} | Get a runtime backup across physical tenants
[**get_runtime_backup_state**](BackupApi.md#get_runtime_backup_state) | **GET** /backups/runtime/state | Get runtime backup state
[**get_runtime_backup_state_as_cluster_admin**](BackupApi.md#get_runtime_backup_state_as_cluster_admin) | **GET** /cluster/v2/backups/runtime/state | Get runtime backup state across physical tenants
[**list_history_backups**](BackupApi.md#list_history_backups) | **GET** /backups/history | List history backups
[**list_history_backups_as_cluster_admin**](BackupApi.md#list_history_backups_as_cluster_admin) | **GET** /cluster/v2/backups/history | List history backups across physical tenants
[**list_runtime_backups**](BackupApi.md#list_runtime_backups) | **GET** /backups/runtime | List runtime backups
[**list_runtime_backups_as_cluster_admin**](BackupApi.md#list_runtime_backups_as_cluster_admin) | **GET** /cluster/v2/backups/runtime | List runtime backups across physical tenants
[**sync_runtime_backup_state**](BackupApi.md#sync_runtime_backup_state) | **POST** /backups/runtime/state/sync | Force-write runtime backup state
[**sync_runtime_backup_state_as_cluster_admin**](BackupApi.md#sync_runtime_backup_state_as_cluster_admin) | **POST** /cluster/v2/backups/runtime/state/sync | Force-write runtime backup state across physical tenants
[**take_history_backup**](BackupApi.md#take_history_backup) | **POST** /backups/history | Take a history backup
[**take_history_backup_as_cluster_admin**](BackupApi.md#take_history_backup_as_cluster_admin) | **POST** /cluster/v2/backups/history | Take a history backup on one or every physical tenant
[**take_runtime_backup**](BackupApi.md#take_runtime_backup) | **POST** /backups/runtime | Take a runtime backup
[**take_runtime_backup_as_cluster_admin**](BackupApi.md#take_runtime_backup_as_cluster_admin) | **POST** /cluster/v2/backups/runtime | Take a runtime backup on one or every physical tenant



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


## delete_history_backup_as_cluster_admin

> delete_history_backup_as_cluster_admin(backup_id, physical_tenant_id)
Delete a history backup across physical tenants

Deletes the history backup with the given id from every physical tenant of the cluster, or from the one named by `physicalTenantId`. A tenant that does not hold the backup has already reached the requested end state, so it counts as deleted rather than as a failure.  The request is all-or-nothing: a physical tenant the backup cannot be deleted from fails the whole request, and the deletions that already succeeded on other tenants are not undone. Narrow the request with `physicalTenantId` to delete from the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. Use `DELETE /v2/backups/history/{backupId}` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

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


## delete_runtime_backup_as_cluster_admin

> delete_runtime_backup_as_cluster_admin(backup_id, physical_tenant_id)
Delete a runtime backup across physical tenants

Deletes the runtime backup with the given id from every physical tenant of the cluster, or from the one named by `physicalTenantId`. A tenant that does not hold the backup has already reached the requested end state, so it counts as deleted rather than as a failure — the same as deleting an unknown backup id through the per-physical-tenant endpoint.  The request is all-or-nothing: a physical tenant the backup cannot be deleted from fails the whole request, and the deletions that already succeeded on other tenants are not undone. Narrow the request with `physicalTenantId` to delete from the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `DELETE /v2/backups/runtime/{backupId}` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

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


## delete_runtime_backup_state_as_cluster_admin

> delete_runtime_backup_state_as_cluster_admin(physical_tenant_id)
Delete runtime backup state across physical tenants

Resets the runtime backup state of every partition of every physical tenant of the cluster, or of the one named by `physicalTenantId`, clearing all checkpoint info, backup info, checkpoint metadata, and backup ranges. Used when switching backup stores.  The request is all-or-nothing: a physical tenant whose state cannot be reset fails the whole request, and the resets that already succeeded on other tenants are not undone. Narrow the request with `physicalTenantId` to reset the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `DELETE /v2/backups/runtime/state` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

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


## get_history_backup_as_cluster_admin

> models::ClusterHistoryBackupInfo get_history_backup_as_cluster_admin(backup_id, physical_tenant_id)
Get a history backup across physical tenants

Reports what every physical tenant of the cluster, or the one named by `physicalTenantId`, holds for the given backup id. There is no aggregated cluster-level state: a tenant that was reached and does not hold this backup reports `NOT_FOUND`, which is a successful observation rather than a failure.  The request is all-or-nothing: a physical tenant whose state cannot be read fails the whole request. Narrow the request with `physicalTenantId` to read the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. Use `GET /v2/backups/history/{backupId}` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

[**models::ClusterHistoryBackupInfo**](ClusterHistoryBackupInfo.md)

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


## get_runtime_backup_as_cluster_admin

> models::ClusterRuntimeBackupInfo get_runtime_backup_as_cluster_admin(backup_id, physical_tenant_id)
Get a runtime backup across physical tenants

Reports what every physical tenant of the cluster, or the one named by `physicalTenantId`, holds for the given backup id, plus the state aggregated over all of them. A tenant that was reached and does not hold this backup reports `DOES_NOT_EXIST`, which is a successful observation rather than a failure — so a backup only some tenants hold aggregates to `INCOMPLETE`, the same way a backup only some partitions hold does within one tenant.  The request is all-or-nothing: a physical tenant whose state cannot be read fails the whole request. Narrow the request with `physicalTenantId` to read the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `GET /v2/backups/runtime/{backupId}` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

[**models::ClusterRuntimeBackupInfo**](ClusterRuntimeBackupInfo.md)

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


## get_runtime_backup_state_as_cluster_admin

> models::ClusterRuntimeBackupState get_runtime_backup_state_as_cluster_admin(physical_tenant_id)
Get runtime backup state across physical tenants

Reports the checkpoint and backup state of every partition of every physical tenant of the cluster, or of the one named by `physicalTenantId`, grouped by physical tenant. Checkpoint ids and log positions only mean anything within one physical tenant's partitions, so nothing is aggregated across tenants.  The request is all-or-nothing: a physical tenant whose state cannot be read fails the whole request rather than contributing an empty section, which an operator making a delete or restore decision could not tell apart from \"nothing to report yet\". Narrow the request with `physicalTenantId` to read the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `GET /v2/backups/runtime/state` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

[**models::ClusterRuntimeBackupState**](ClusterRuntimeBackupState.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

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


## list_history_backups_as_cluster_admin

> Vec<models::ClusterHistoryBackupInfo> list_history_backups_as_cluster_admin(physical_tenant_id, prefix, verbose)
List history backups across physical tenants

Lists the history backups of every physical tenant of the cluster, or of the one named by `physicalTenantId`, grouped by backup id. A backup id that only some physical tenants hold is a supported outcome rather than a degraded one, so only the tenants that hold it are listed under it.  The request is all-or-nothing: a physical tenant whose backups cannot be read fails the whole request rather than silently dropping out of the listing. Narrow the request with `physicalTenantId` to list the backups of the tenants that can still be read.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. Use `GET /v2/backups/history` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |
**prefix** | Option<**String**> | A prefix that backup ids must match, ending in a single '*'. If omitted, all backups are returned.  |  |
**verbose** | Option<**bool**> | Whether to ask the secondary storage for snapshot-level detail. Setting this to `false` makes the query cheaper, but the store then reports neither snapshot state nor start time, so both the per-snapshot `details` and the per-tenant `state` are incomplete and the listing order is unspecified.  |  |[default to true]

### Return type

[**Vec<models::ClusterHistoryBackupInfo>**](ClusterHistoryBackupInfo.md)

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


## list_runtime_backups_as_cluster_admin

> Vec<models::ClusterRuntimeBackupInfo> list_runtime_backups_as_cluster_admin(physical_tenant_id, prefix)
List runtime backups across physical tenants

Lists the runtime backups of every physical tenant of the cluster, or of the one named by `physicalTenantId`, grouped by backup id. Every group reports every targeted tenant, including the ones holding nothing for that id, so a backup only some tenants hold aggregates to `INCOMPLETE` here exactly as it does when looked up directly — the state of a listed group can be trusted to say whether the cluster can be restored from it. A backup id that only some physical tenants hold is a supported outcome rather than a degraded one; tenants that generate their own backup ids never share one, so in that mode each backup forms its own group and the other tenants report `DOES_NOT_EXIST` under it.  The request is all-or-nothing: a physical tenant whose backups cannot be read fails the whole request rather than silently dropping out of the listing. Narrow the request with `physicalTenantId` to list the backups of the tenants that can still be read.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `GET /v2/backups/runtime` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |
**prefix** | Option<**String**> | A prefix that backup ids must match, ending in a single '*'. If omitted, all backups are returned.  |  |

### Return type

[**Vec<models::ClusterRuntimeBackupInfo>**](ClusterRuntimeBackupInfo.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

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


## sync_runtime_backup_state_as_cluster_admin

> models::ClusterRuntimeBackupState sync_runtime_backup_state_as_cluster_admin(physical_tenant_id)
Force-write runtime backup state across physical tenants

Force-writes the checkpoint and backup metadata of every partition of every physical tenant of the cluster, or of the one named by `physicalTenantId`, to that tenant's backup store, independent of any backup being taken or confirmed, and returns the updated state per physical tenant.  The request is all-or-nothing: a physical tenant whose metadata cannot be written fails the whole request, and the writes that already succeeded on other tenants are not undone. The operation is idempotent, so retrying the same call is the correct remedy. Narrow the request with `physicalTenantId` to write the tenants that can still be reached.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `POST /v2/backups/runtime/state/sync` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

[**models::ClusterRuntimeBackupState**](ClusterRuntimeBackupState.md)

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


## take_history_backup_as_cluster_admin

> models::ClusterTakeHistoryBackupResponse take_history_backup_as_cluster_admin(take_history_backup_request, physical_tenant_id)
Take a history backup on one or every physical tenant

Triggers a history backup on every physical tenant of the cluster, or on the one named by `physicalTenantId`. Every targeted tenant uses the same caller-supplied `backupId`, but the backups are independent: they are neither coordinated nor rolled back together.  The request is all-or-nothing: the `backupId` is checked on every targeted tenant before any snapshot is scheduled, so a tenant that already holds this id, or that cannot be reached, fails the whole request and no backup is started anywhere. There is no aggregated cluster-level state in the response.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Only available on clusters whose secondary storage is Elasticsearch or OpenSearch. Use `POST /v2/backups/history` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**take_history_backup_request** | [**TakeHistoryBackupRequest**](TakeHistoryBackupRequest.md) |  | [required] |
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |

### Return type

[**models::ClusterTakeHistoryBackupResponse**](ClusterTakeHistoryBackupResponse.md)

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


## take_runtime_backup_as_cluster_admin

> models::ClusterTakeRuntimeBackupResponse take_runtime_backup_as_cluster_admin(physical_tenant_id, take_runtime_backup_request)
Take a runtime backup on one or every physical tenant

Triggers a runtime backup on every physical tenant of the cluster, or on the one named by `physicalTenantId`. A cluster-wide backup is a set of independent per-tenant backups, not an atomic snapshot of the cluster: they are neither coordinated nor rolled back together, and each tenant stores its own, so the same `backupId` can be used for all of them.  Every targeted physical tenant must be in the same backup-id mode. `backupId` must be omitted when every targeted tenant generates its own ids (because continuous backups and/or a backup or checkpoint schedule is enabled for it), and is required when none of them does. A cluster whose targeted tenants mix the two modes is rejected with 400 and has to be driven one tenant at a time through `POST /v2/backups/runtime`. In generated-id mode each tenant generates its own id, so the response reports an id per physical tenant rather than one for the cluster.  The trigger is all-or-error, and never silent about a partial trigger: if any targeted tenant cannot be triggered the response carries an error status, but its body still lists every targeted tenant — which ones were triggered, under which `backupId` to monitor or delete them, and why the others failed. Nothing is rolled back, so the backups that were triggered keep running and have to be deleted explicitly. A request rejected before any tenant was triggered answers with a problem detail instead, and nothing is running.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `POST /v2/backups/runtime` to act as a single physical tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**physical_tenant_id** | Option<**String**> | The physical tenant to apply the change to. When omitted, or when passed with an empty value, the change is applied to every physical tenant of the cluster. |  |
**take_runtime_backup_request** | Option<[**TakeRuntimeBackupRequest**](TakeRuntimeBackupRequest.md)> |  |  |

### Return type

[**models::ClusterTakeRuntimeBackupResponse**](ClusterTakeRuntimeBackupResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

