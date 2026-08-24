# ClusterRebalanceOperationPartition

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**partition_id** | **i32** | The unique ID of this partition, within its physical tenant. | 
**physical_tenant_id** | **String** | The partition group this partition belongs to. | 
**current_leader** | Option<**String**> | The leader last observed by this rebalance, or absent if there was no leader. | 
**desired_leader** | **String** | The leader selected when this rebalance was planned. | 
**progress** | **Progress** | Where this rebalance has reached for the partition. (enum: PENDING, TRANSFERRING, COMPLETED) | 
**result** | Option<**Result**> | The terminal outcome, present only when progress is COMPLETED. (enum: TRANSFERRED, ALREADY_LEADER, NOT_MEMBER, NOT_REPLICATING, UNREACHABLE, NOT_COORDINATOR, STALE_CONFIGURATION, TRANSFER_IN_PROGRESS, LAG_TOO_HIGH, LEADER_INITIALIZING, CONFIGURATION_CHANGE_IN_PROGRESS, PAUSE_FAILED, REPLICATION_TIMED_OUT, TIMEOUT_NOW_EXHAUSTED, LEADER_CHANGED, NO_LEADER, NO_RESPONSE, CANCELLED) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


