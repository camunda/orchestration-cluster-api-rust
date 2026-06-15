# BrokerInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**node_id** | **i32** | The unique (within a cluster) node ID for the broker. | 
**host** | **String** | The hostname for reaching the broker. | 
**port** | **i32** | The port for reaching the broker. | 
**partitions** | [**Vec<models::Partition>**](Partition.md) | A list of partitions managed or replicated on this broker. | 
**version** | **String** | The broker version. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


