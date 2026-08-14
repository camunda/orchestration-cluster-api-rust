# \ClusterApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_cluster_status**](ClusterApi.md#get_cluster_status) | **GET** /cluster/v2/status | Get the status of the whole cluster
[**get_status**](ClusterApi.md#get_status) | **GET** /status | Get physical tenant status
[**get_topology**](ClusterApi.md#get_topology) | **GET** /topology | Get cluster topology



## get_cluster_status

> models::ClusterStatusResponse get_cluster_status()
Get the status of the whole cluster

Checks the health status of the whole cluster, aggregated over all physical tenants. Returns `HEALTHY` when every physical tenant is healthy, `DOWN` when no physical tenant can process work, and `DEGRADED` in every other case. No per-tenant detail is reported; use `GET /cluster/v2/topology` for that.  This endpoint is public and requires no authentication, unlike `PATCH /cluster/v2/mode` below, which needs cluster-admin credentials.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ClusterStatusResponse**](ClusterStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_status

> get_status()
Get physical tenant status

Checks the health status of the default physical tenant by verifying if there's at least one partition of its group with a healthy leader. This endpoint is scoped to the default physical tenant only: it is available unprefixed and at `/physical-tenants/default/v2/status`, but not for any other physical tenant id (`/physical-tenants/{id}/v2/status` returns 404 for every other id, whether or not a physical tenant with that id exists). On a cluster with only the default physical tenant this endpoint answers the same question as `/cluster/v2/status`, though not with the same response: `/cluster/v2/status` reports its status in a body and so also distinguishes a degraded tenant from a healthy one. Use `/cluster/v2/status` for the aggregated status of the whole cluster, or `/physical-tenants/{id}/v2/topology` for the health of a specific physical tenant's partitions.

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

