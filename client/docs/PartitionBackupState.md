# PartitionBackupState

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**checkpoint_id** | **i64** | The id of the checkpoint this backup is based on. | 
**checkpoint_type** | [**models::BackupType**](BackupType.md) | The type of the backup. | 
**partition_id** | Option<**i32**> | The id of the partition. Omitted when nested inside a backup range's `start`/`end`, where the partition is already identified by the enclosing range.  | 
**checkpoint_position** | **i64** | The log position of the checkpoint this backup is based on. | 
**first_log_position** | **i64** | The first log position included in this backup. | 
**checkpoint_timestamp** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp at which the checkpoint was created. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


