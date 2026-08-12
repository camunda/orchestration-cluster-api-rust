# BackupInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [readonly]
**state** | [**models::StateCode**](StateCode.md) | The aggregated state of the backup. | [readonly]
**failure_reason** | Option<**String**> | Reason for failure if the state is 'FAILED'. | 
**details** | [**Vec<models::PartitionBackupInfo>**](PartitionBackupInfo.md) | Detailed status of the backup per partition. Always contains every partition of the physical tenant.  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


