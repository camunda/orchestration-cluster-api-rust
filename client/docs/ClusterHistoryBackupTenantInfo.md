# ClusterHistoryBackupTenantInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**physical_tenant_id** | **String** | The id of the physical tenant. | 
**state** | [**models::ClusterHistoryBackupTenantState**](ClusterHistoryBackupTenantState.md) | The state of the backup on this physical tenant. | 
**failure_reason** | Option<**String**> | Reason for failure if the state is 'FAILED'. | 
**details** | [**Vec<models::HistoryBackupSnapshotInfo>**](HistoryBackupSnapshotInfo.md) | Detailed status of the backup per snapshot on this physical tenant. Empty when the tenant does not hold the backup. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


