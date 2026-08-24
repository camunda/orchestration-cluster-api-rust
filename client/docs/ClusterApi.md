# \ClusterApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_cluster_rebalance**](ClusterApi.md#cancel_cluster_rebalance) | **DELETE** /cluster/v2/rebalance | Stop the running rebalance
[**get_cluster_rebalance**](ClusterApi.md#get_cluster_rebalance) | **GET** /cluster/v2/rebalance | Report the cluster's current leadership balance
[**get_cluster_status**](ClusterApi.md#get_cluster_status) | **GET** /cluster/v2/status | Get the status of the whole cluster
[**get_cluster_topology**](ClusterApi.md#get_cluster_topology) | **GET** /cluster/v2/topology | Get the topology of the whole cluster
[**get_status**](ClusterApi.md#get_status) | **GET** /status | Get physical tenant status
[**get_topology**](ClusterApi.md#get_topology) | **GET** /topology | Get cluster topology
[**trigger_cluster_rebalance**](ClusterApi.md#trigger_cluster_rebalance) | **POST** /cluster/v2/rebalance | Trigger a cluster-wide leadership rebalance



## cancel_cluster_rebalance

> models::RebalanceCancellationResponse cancel_cluster_rebalance()
Stop the running rebalance

Asks the running rebalance to stop once the transfer in flight has finished. Partitions already transferred keep their new leaders, and those the rebalance had not yet reached keep their current ones.  Cancellation requests are idempotent and always accepted. The `wasRunning` response field can be used to distinguish a cancellation that found a running rebalance from one that did not.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::RebalanceCancellationResponse**](RebalanceCancellationResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_cluster_rebalance

> models::ClusterBalanceResponse get_cluster_rebalance()
Report the cluster's current leadership balance

Reports whether the cluster is currently balanced, the current leadership state of every partition, and what became of the last rebalance to finish. The last completed rebalance is held in memory by the coordinating broker, so none will be reported if the coordinator has moved or restarted since the last rebalance.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ClusterBalanceResponse**](ClusterBalanceResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## get_cluster_topology

> models::ClusterTopologyResponse get_cluster_topology()
Get the topology of the whole cluster

Obtains the topology of the whole cluster, aggregated over all physical tenants. Cluster-level information is reported once; partition layout, replication and per-partition role, health and state are reported per physical tenant.  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here. Use `GET /v2/topology` for the topology of a single physical tenant.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ClusterTopologyResponse**](ClusterTopologyResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

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


## trigger_cluster_rebalance

> models::ClusterBalanceResponse trigger_cluster_rebalance(dry_run, cluster_rebalance_request)
Trigger a cluster-wide leadership rebalance

Transfers leadership of every partition that is not led by its highest-priority replica towards that replica, one partition at a time. Returns as soon as the rebalance has been accepted (poll `GET /cluster/v2/rebalance` to monitor progress).  Each rebalance can specify overrides for the configured rebalance settings (e.g. maximum replication lag to allow). An absent request body means \"use the configured settings\".  Requires the cluster-admin security chain. Although this operation lists `bearerAuth` / `basicAuth` like the rest of the Orchestration Cluster API, it does not accept an Orchestration Cluster user's credentials — only the separate cluster-admin credentials are valid here.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dry_run** | Option<**bool**> | If true, report the plan the rebalance would carry out without pausing any partition or transferring any leadership. |  |[default to false]
**cluster_rebalance_request** | Option<[**ClusterRebalanceRequest**](ClusterRebalanceRequest.md)> |  |  |

### Return type

[**models::ClusterBalanceResponse**](ClusterBalanceResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

