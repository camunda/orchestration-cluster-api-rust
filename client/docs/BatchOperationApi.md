# \BatchOperationApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_batch_operation**](BatchOperationApi.md#cancel_batch_operation) | **POST** /batch-operations/{batchOperationKey}/cancellation | Cancel Batch operation
[**get_batch_operation**](BatchOperationApi.md#get_batch_operation) | **GET** /batch-operations/{batchOperationKey} | Get batch operation
[**resume_batch_operation**](BatchOperationApi.md#resume_batch_operation) | **POST** /batch-operations/{batchOperationKey}/resumption | Resume Batch operation
[**search_batch_operation_items**](BatchOperationApi.md#search_batch_operation_items) | **POST** /batch-operation-items/search | Search batch operation items
[**search_batch_operations**](BatchOperationApi.md#search_batch_operations) | **POST** /batch-operations/search | Search batch operations
[**suspend_batch_operation**](BatchOperationApi.md#suspend_batch_operation) | **POST** /batch-operations/{batchOperationKey}/suspension | Suspend Batch operation



## cancel_batch_operation

> cancel_batch_operation(batch_operation_key)
Cancel Batch operation

Cancels a running batch operation. This is done asynchronously, the progress can be tracked using the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_operation_key** | **String** | The key (or operate legacy ID) of the batch operation. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_batch_operation

> models::BatchOperationResponse get_batch_operation(batch_operation_key)
Get batch operation

Get batch operation by key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_operation_key** | **String** | The key (or operate legacy ID) of the batch operation. | [required] |

### Return type

[**models::BatchOperationResponse**](BatchOperationResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resume_batch_operation

> resume_batch_operation(batch_operation_key)
Resume Batch operation

Resumes a suspended batch operation. This is done asynchronously, the progress can be tracked using the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_operation_key** | **String** | The key (or operate legacy ID) of the batch operation. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_batch_operation_items

> models::BatchOperationItemSearchQueryResult search_batch_operation_items(batch_operation_item_search_query)
Search batch operation items

Search for batch operation items based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_operation_item_search_query** | Option<[**BatchOperationItemSearchQuery**](BatchOperationItemSearchQuery.md)> |  |  |

### Return type

[**models::BatchOperationItemSearchQueryResult**](BatchOperationItemSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_batch_operations

> models::BatchOperationSearchQueryResult search_batch_operations(batch_operation_search_query)
Search batch operations

Search for batch operations based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_operation_search_query** | Option<[**BatchOperationSearchQuery**](BatchOperationSearchQuery.md)> |  |  |

### Return type

[**models::BatchOperationSearchQueryResult**](BatchOperationSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## suspend_batch_operation

> suspend_batch_operation(batch_operation_key)
Suspend Batch operation

Suspends a running batch operation. This is done asynchronously, the progress can be tracked using the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_operation_key** | **String** | The key (or operate legacy ID) of the batch operation. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

