# ClusterRestorePartitionRestoreOperation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operation** | **String** | The type of the operation. | 
**broker_id** | **String** | The ID of the broker that applies the operation, including its zone if it belongs to one. | 
**partition_id** | **i32** | The partition the operation restores. | 
**backup_ids** | **Vec<i64>** | The IDs of the backups the partition is restored from. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


