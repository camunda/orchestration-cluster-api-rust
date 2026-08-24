# ClusterBrokerInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**broker_id** | **String** | The unique (within a cluster) broker identifier. When the cluster is not zoned, then it's a string that represents the nodeId (an integer). When the cluster is zoned, instead, it's of the form \"$zoneName_$nodeId\", providing uniqueness even across zones.  | 
**host** | **String** | The hostname for reaching the broker. | 
**port** | **i32** | The port for reaching the broker. | 
**version** | **String** | The broker version. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


