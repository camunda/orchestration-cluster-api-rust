# \ClusterApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_status**](ClusterApi.md#get_status) | **GET** /status | Get cluster status
[**get_topology**](ClusterApi.md#get_topology) | **GET** /topology | Get cluster topology



## get_status

> get_status()
Get cluster status

Checks the health status of the cluster by verifying if there's at least one partition with a healthy leader.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_topology

> models::TopologyResponse get_topology()
Get cluster topology

Obtains the current topology of the cluster the gateway is part of.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::TopologyResponse**](TopologyResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

