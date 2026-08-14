# HistoryBackupInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | [readonly]
**state** | [**models::HistoryBackupStateCode**](HistoryBackupStateCode.md) | The aggregated state of the backup. | [readonly]
**failure_reason** | Option<**String**> | Reason for failure if the state is 'FAILED'. | 
**details** | [**Vec<models::HistoryBackupSnapshotInfo>**](HistoryBackupSnapshotInfo.md) | Detailed status of the backup per snapshot. Always lists every snapshot found for the backup; when the backup was read without snapshot detail, each entry carries only its name.  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


