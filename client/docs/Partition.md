# Partition

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**partition_id** | **i32** | The unique ID of this partition. | 
**role** | **Role** | Describes the Raft role of the broker for a given partition. (enum: leader, follower, inactive) | 
**health** | **Health** | Describes the current health of the partition. (enum: healthy, unhealthy, dead) | 
**state** | **State** | Describes the current operational state of the partition within the cluster configuration.  (enum: unknown, joining, active, leaving, recovering) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


