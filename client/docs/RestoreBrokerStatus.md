# RestoreBrokerStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**broker_id** | **String** | The ID of the broker, including its zone if it belongs to one. | 
**partitions_restored** | **i32** | The number of the broker's partitions that have been restored so far. | 
**partitions_to_restore** | **i32** | The total number of the broker's partitions to restore. | 
**partitions** | [**Vec<models::RestorePartitionStatus>**](RestorePartitionStatus.md) | The per-partition restore status for this broker. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


