# PhysicalTenantTopology

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**physical_tenant_id** | **String** | The id of the physical tenant. | 
**partitions_count** | **i32** | The number of partitions spread across this physical tenant. | 
**replication_factor** | **i32** | The configured replication factor for this physical tenant. | 
**last_completed_change_id** | **String** | ID of the last completed change of this physical tenant. | 
**brokers** | [**Vec<models::PhysicalTenantBrokerTopology>**](PhysicalTenantBrokerTopology.md) | The brokers holding partitions of this physical tenant. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


