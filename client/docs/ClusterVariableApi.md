# \ClusterVariableApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_global_cluster_variable**](ClusterVariableApi.md#create_global_cluster_variable) | **POST** /cluster-variables/global | Create a global-scoped cluster variable
[**create_tenant_cluster_variable**](ClusterVariableApi.md#create_tenant_cluster_variable) | **POST** /cluster-variables/tenants/{tenantId} | Create a tenant-scoped cluster variable
[**delete_global_cluster_variable**](ClusterVariableApi.md#delete_global_cluster_variable) | **DELETE** /cluster-variables/global/{name} | Delete a global-scoped cluster variable
[**delete_tenant_cluster_variable**](ClusterVariableApi.md#delete_tenant_cluster_variable) | **DELETE** /cluster-variables/tenants/{tenantId}/{name} | Delete a tenant-scoped cluster variable
[**get_global_cluster_variable**](ClusterVariableApi.md#get_global_cluster_variable) | **GET** /cluster-variables/global/{name} | Get a global-scoped cluster variable
[**get_tenant_cluster_variable**](ClusterVariableApi.md#get_tenant_cluster_variable) | **GET** /cluster-variables/tenants/{tenantId}/{name} | Get a tenant-scoped cluster variable
[**search_cluster_variables**](ClusterVariableApi.md#search_cluster_variables) | **POST** /cluster-variables/search | 
[**update_global_cluster_variable**](ClusterVariableApi.md#update_global_cluster_variable) | **PUT** /cluster-variables/global/{name} | Update a global-scoped cluster variable
[**update_tenant_cluster_variable**](ClusterVariableApi.md#update_tenant_cluster_variable) | **PUT** /cluster-variables/tenants/{tenantId}/{name} | Update a tenant-scoped cluster variable



## create_global_cluster_variable

> models::ClusterVariableResult create_global_cluster_variable(create_cluster_variable_request)
Create a global-scoped cluster variable

Create a global-scoped cluster variable.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_cluster_variable_request** | [**CreateClusterVariableRequest**](CreateClusterVariableRequest.md) |  | [required] |

### Return type

[**models::ClusterVariableResult**](ClusterVariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_tenant_cluster_variable

> models::ClusterVariableResult create_tenant_cluster_variable(tenant_id, create_cluster_variable_request)
Create a tenant-scoped cluster variable

Create a new cluster variable for the given tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The tenant ID | [required] |
**create_cluster_variable_request** | [**CreateClusterVariableRequest**](CreateClusterVariableRequest.md) |  | [required] |

### Return type

[**models::ClusterVariableResult**](ClusterVariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_global_cluster_variable

> delete_global_cluster_variable(name)
Delete a global-scoped cluster variable

Delete a global-scoped cluster variable.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | The name of the cluster variable | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_tenant_cluster_variable

> delete_tenant_cluster_variable(tenant_id, name)
Delete a tenant-scoped cluster variable

Delete a tenant-scoped cluster variable.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The tenant ID | [required] |
**name** | **String** | The name of the cluster variable | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_global_cluster_variable

> models::ClusterVariableResult get_global_cluster_variable(name)
Get a global-scoped cluster variable

Get a global-scoped cluster variable.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | The name of the cluster variable | [required] |

### Return type

[**models::ClusterVariableResult**](ClusterVariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant_cluster_variable

> models::ClusterVariableResult get_tenant_cluster_variable(tenant_id, name)
Get a tenant-scoped cluster variable

Get a tenant-scoped cluster variable.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The tenant ID | [required] |
**name** | **String** | The name of the cluster variable | [required] |

### Return type

[**models::ClusterVariableResult**](ClusterVariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_cluster_variables

> models::ClusterVariableSearchQueryResult search_cluster_variables(truncate_values, cluster_variable_search_query_request)


Search for cluster variables based on given criteria. By default, long variable values in the response are truncated.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**truncate_values** | Option<**bool**> | When true (default), long variable values in the response are truncated. When false, full variable values are returned. |  |
**cluster_variable_search_query_request** | Option<[**ClusterVariableSearchQueryRequest**](ClusterVariableSearchQueryRequest.md)> |  |  |

### Return type

[**models::ClusterVariableSearchQueryResult**](ClusterVariableSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_global_cluster_variable

> models::ClusterVariableResult update_global_cluster_variable(name, update_cluster_variable_request)
Update a global-scoped cluster variable

Updates the value of an existing global cluster variable. The variable must exist, otherwise a 404 error is returned. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | The name of the cluster variable | [required] |
**update_cluster_variable_request** | [**UpdateClusterVariableRequest**](UpdateClusterVariableRequest.md) |  | [required] |

### Return type

[**models::ClusterVariableResult**](ClusterVariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant_cluster_variable

> models::ClusterVariableResult update_tenant_cluster_variable(tenant_id, name, update_cluster_variable_request)
Update a tenant-scoped cluster variable

Updates the value of an existing tenant-scoped cluster variable. The variable must exist, otherwise a 404 error is returned. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The tenant ID | [required] |
**name** | **String** | The name of the cluster variable | [required] |
**update_cluster_variable_request** | [**UpdateClusterVariableRequest**](UpdateClusterVariableRequest.md) |  | [required] |

### Return type

[**models::ClusterVariableResult**](ClusterVariableResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

