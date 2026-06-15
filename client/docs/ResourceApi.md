# \ResourceApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_deployment**](ResourceApi.md#create_deployment) | **POST** /deployments | Deploy resources
[**delete_resource**](ResourceApi.md#delete_resource) | **POST** /resources/{resourceKey}/deletion | Delete resource
[**get_resource**](ResourceApi.md#get_resource) | **GET** /resources/{resourceKey} | Get resource
[**get_resource_content**](ResourceApi.md#get_resource_content) | **GET** /resources/{resourceKey}/content | Get RPA resource content (deprecated)
[**get_resource_content_binary**](ResourceApi.md#get_resource_content_binary) | **GET** /resources/{resourceKey}/content/binary | Get resource content as binary
[**search_resources**](ResourceApi.md#search_resources) | **POST** /resources/search | Search resources



## create_deployment

> models::DeploymentResult create_deployment(resources, tenant_id)
Deploy resources

Deploys one or more resources, including BPMN processes, DMN decision models, forms, RPA resources, and generic files. A deployment can contain any file type. Files that are not interpreted as BPMN, DMN, form, or RPA resources are stored as deployable generic resources in the engine. This is an atomic call, i.e. either all resources are deployed or none of them are. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**resources** | [**Vec<std::path::PathBuf>**](Std__path__PathBuf.md) | The binary data to create the deployment resources. It is possible to have more than one form part with different form part names for the binary data to create a deployment.  | [required] |
**tenant_id** | Option<**String**> | The unique identifier of the tenant. |  |

### Return type

[**models::DeploymentResult**](DeploymentResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_resource

> models::DeleteResourceResponse delete_resource(resource_key, delete_resource_request)
Delete resource

Deletes a deployed resource. This can be a process definition, decision requirements definition, or form definition deployed using the deploy resources endpoint. Specify the resource you want to delete in the `resourceKey` parameter.  Once a resource has been deleted it cannot be recovered. If the resource needs to be available again, a new deployment of the resource is required.  By default, only the resource itself is deleted from the runtime state. To also delete the historic data associated with a resource, set the `deleteHistory` flag in the request body to `true`. The historic data is deleted asynchronously via a batch operation. The details of the created batch operation are included in the response. Note that history deletion is only supported for process resources; for other resource types this flag is ignored and no history will be deleted.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**resource_key** | **ResourceKey** | The key of the resource to delete. This can be the key of a process definition, the key of a decision requirements definition or the key of a form definition  | [required] |
**delete_resource_request** | Option<[**DeleteResourceRequest**](DeleteResourceRequest.md)> |  |  |

### Return type

[**models::DeleteResourceResponse**](DeleteResourceResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_resource

> models::ResourceResult get_resource(resource_key)
Get resource

Returns a deployed resource. :::info This endpoint does not return BPMN process definitions, DMN decision definitions, or form resources. To query BPMN process definitions or DMN decision definitions, use their respective APIs. ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**resource_key** | **ResourceKey** | The unique key identifying the resource. | [required] |

### Return type

[**models::ResourceResult**](ResourceResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_resource_content

> std::collections::HashMap<String, serde_json::Value> get_resource_content(resource_key)
Get RPA resource content (deprecated)

**Deprecated** — use `/resources/{resourceKey}/content/binary` instead, which supports all resource types and returns content as binary (octet-stream).  Returns the content of a deployed RPA resource as JSON. :::info This endpoint only supports RPA resources. For generic resource content in binary format, use the `/resources/{resourceKey}/content/binary` endpoint. ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**resource_key** | **ResourceKey** | The unique key identifying the RPA resource. | [required] |

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_resource_content_binary

> std::path::PathBuf get_resource_content_binary(resource_key)
Get resource content as binary

Returns the content of a deployed resource in binary format (octet-stream). :::info This endpoint does not return BPMN process definitions, DMN decision definitions, or form resources. To query BPMN process definitions or DMN decision definitions, use their respective APIs. ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**resource_key** | **ResourceKey** | The unique key identifying the resource. | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_resources

> models::ResourceSearchQueryResult search_resources(resource_search_query)
Search resources

Search for deployed resources based on given criteria. :::info This endpoint does not return BPMN process definitions, DMN decision definitions, or form resources. To query BPMN process definitions or DMN decision definitions, use their respective search APIs. ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**resource_search_query** | Option<[**ResourceSearchQuery**](ResourceSearchQuery.md)> |  |  |

### Return type

[**models::ResourceSearchQueryResult**](ResourceSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

