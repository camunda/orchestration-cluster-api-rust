# ClusterHistoryBackupInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | 
**physical_tenants** | [**Vec<models::ClusterHistoryBackupTenantInfo>**](ClusterHistoryBackupTenantInfo.md) | What each physical tenant reports for this backup id, ordered by physical tenant id. When looking a backup id up directly, every targeted tenant is listed, including the ones reporting `NOT_FOUND`. Within a listing, only the tenants that hold the id are listed. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


