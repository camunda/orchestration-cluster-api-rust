# ClusterRunningRebalance

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**rebalance_id** | **i64** | The ID of this rebalance. | 
**partitions** | [**Vec<models::ClusterRebalanceOperationPartition>**](ClusterRebalanceOperationPartition.md) | Every partition in the rebalance plan and its progress within this rebalance. | 
**started_at** | **chrono::DateTime<chrono::FixedOffset>** | When this rebalance was created. | 
**dry_run** | **bool** | Whether this rebalance is a dry run. | 
**cancel_requested** | **bool** | Whether cancellation has been requested. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


