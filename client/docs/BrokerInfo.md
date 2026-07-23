# BrokerInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**node_id** | **i32** | The node ID for the broker. The uniqueness of this identifier depends if the cluster is zone-aware or not. - non zone-aware: (default) nodeId is unique across the cluster - zone-aware:  (opt-in) nodeId is unique only within its zone. If you are migrating to a zone aware cluster, you must use `brokerId` instead. This property is deprecated, as it's been replaced by `brokerId`.  | 
**broker_id** | **String** | The unique (within a cluster) broker identifier. When the cluster is not zoned, then it's a string that represents the nodeId (an integer). When the cluster is zoned, instead, it's of the form \"$zoneName_$nodeId\", providing uniqueness even across zones.  | 
**host** | **String** | The hostname for reaching the broker. | 
**port** | **i32** | The port for reaching the broker. | 
**partitions** | [**Vec<models::Partition>**](Partition.md) | A list of partitions managed or replicated on this broker. | 
**version** | **String** | The broker version. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


