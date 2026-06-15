# TopologyResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**brokers** | [**Vec<models::BrokerInfo>**](BrokerInfo.md) | A list of brokers that are part of this cluster. | 
**cluster_id** | Option<**String**> | The cluster Id. | 
**cluster_size** | **i32** | The number of brokers in the cluster. | 
**partitions_count** | **i32** | The number of partitions are spread across the cluster. | 
**replication_factor** | **i32** | The configured replication factor for this cluster. | 
**gateway_version** | **String** | The version of the Zeebe Gateway. | 
**last_completed_change_id** | **String** | ID of the last completed change | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


