# ClusterBalanceResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **State** | The cluster's aggregate balance state as of the time of the request. (enum: BALANCED, BALANCING, UNBALANCED) | 
**partitions** | [**Vec<models::ClusterRebalancePartition>**](ClusterRebalancePartition.md) | The balance state of each partition as of the time of the request. | 
**running_rebalance** | Option<[**models::ClusterRunningRebalance**](ClusterRunningRebalance.md)> | Normally the rebalance currently running, or absent if no rebalance is running. For a dry-run response, this is instead the unexecuted plan of that dry run. | 
**last_completed_rebalance** | Option<[**models::ClusterCompletedRebalance**](ClusterCompletedRebalance.md)> | The last completed non-dry-run rebalance this coordinator finished. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


