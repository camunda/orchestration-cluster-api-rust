# RestorePartitionStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**partition_id** | **i32** | The ID of the partition. | 
**state** | **State** | The restore state of the partition. (enum: PENDING, RESTORING, RESTORED) | 
**backup_ids** | **Vec<i64>** | The IDs of the backups this partition is restored from. | 
**completed_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The time the partition was restored, as an ISO 8601 timestamp; null unless the partition state is `RESTORED`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


