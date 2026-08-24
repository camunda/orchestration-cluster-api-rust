# ClusterRuntimeBackupTakeResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**physical_tenant_id** | **String** | The id of the physical tenant. | 
**backup_id** | Option<**i64**> | The id to monitor or delete this physical tenant's backup by: the id it is running under when `TRIGGERED` — the requested one, or the one the tenant generated when ids are generated — and the requested id to check when `UNKNOWN`. Null when the tenant is known to be running no backup, and also when an `UNKNOWN` tenant generates its own ids, because the id it may be running under was never reported; list that tenant's backups to find it. | 
**outcome** | [**models::ClusterRuntimeBackupTakeOutcome**](ClusterRuntimeBackupTakeOutcome.md) | What this physical tenant did with the trigger. | 
**reason** | Option<**String**> | Why this physical tenant reported no triggered backup. Null when it was triggered. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


