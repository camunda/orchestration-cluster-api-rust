# ClusterRebalancePartition

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**partition_id** | **i32** | The unique ID of this partition, within its physical tenant. | 
**physical_tenant_id** | **String** | The partition group this partition belongs to. Partition IDs are unique only within a group, so this is needed to identify the partition. | 
**current_leader** | Option<**String**> | The broker ID currently leading this partition, or absent if it has no leader. | 
**desired_leader** | **String** | The broker ID the current configuration wants to lead this partition. | 
**state** | **State** | Whether this partition is being actively transferred, unbalanced, or balanced. (enum: TRANSFERRING, UNBALANCED, BALANCED) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


