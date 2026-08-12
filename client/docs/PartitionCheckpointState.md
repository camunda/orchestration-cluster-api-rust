# PartitionCheckpointState

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**checkpoint_id** | **i64** | The id of the checkpoint. | 
**checkpoint_type** | [**models::CheckpointType**](CheckpointType.md) | The type of the checkpoint. | 
**partition_id** | **i32** | The id of the partition. | 
**checkpoint_position** | **i64** | The log position of the checkpoint. | 
**checkpoint_timestamp** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp at which the checkpoint was created. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


