# ClusterTakeHistoryBackupResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**backup_id** | **i64** | The id requested for the backup on every targeted physical tenant. | 
**physical_tenants** | [**Vec<models::ClusterHistoryBackupTakeResult>**](ClusterHistoryBackupTakeResult.md) | The outcome for each targeted physical tenant, ordered by physical tenant id. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


