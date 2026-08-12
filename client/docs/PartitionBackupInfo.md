# PartitionBackupInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**partition_id** | **i32** | The id of the partition. | [readonly]
**state** | [**models::StateCode**](StateCode.md) | The state of the backup on this partition. | [readonly]
**failure_reason** | Option<**String**> | Failure reason if the state is 'FAILED'. | 
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The timestamp at which the backup was started on this partition. | [readonly]
**last_updated_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The timestamp at which the backup was last updated on this partition, e.g. changed state from 'IN_PROGRESS' to 'COMPLETED'.  | [readonly]
**snapshot_id** | Option<**String**> | The id of the snapshot which is included in this backup. | [readonly]
**first_log_position** | Option<**i64**> | The first log position included in this backup. | [readonly]
**checkpoint_position** | Option<**i64**> | The position of the checkpoint for this backup. | [readonly]
**broker_id** | Option<**i32**> | The id of the broker from which the backup was taken for this partition. | [readonly]
**broker_version** | Option<**String**> | The version of the broker from which the backup was taken for this partition.  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


