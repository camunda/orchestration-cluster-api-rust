# ClusterTopologyResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**brokers** | [**Vec<models::ClusterBrokerInfo>**](ClusterBrokerInfo.md) | The brokers that are part of this cluster, across all physical tenants. | 
**cluster_id** | Option<**String**> | The cluster Id. | 
**cluster_size** | **i32** | The number of brokers in the cluster. | 
**gateway_version** | Option<**String**> | The version of the Orchestration Cluster Gateway. | 
**physical_tenants** | [**Vec<models::PhysicalTenantTopology>**](PhysicalTenantTopology.md) | The topology of each physical tenant of this cluster. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


