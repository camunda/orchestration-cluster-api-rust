# \DecisionInstanceApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_decision_instance**](DecisionInstanceApi.md#delete_decision_instance) | **POST** /decision-instances/{decisionEvaluationKey}/deletion | Delete decision instance
[**delete_decision_instances_batch_operation**](DecisionInstanceApi.md#delete_decision_instances_batch_operation) | **POST** /decision-instances/deletion | Delete decision instances (batch)
[**get_decision_instance**](DecisionInstanceApi.md#get_decision_instance) | **GET** /decision-instances/{decisionEvaluationInstanceKey} | Get decision instance
[**search_decision_instances**](DecisionInstanceApi.md#search_decision_instances) | **POST** /decision-instances/search | Search decision instances



## delete_decision_instance

> delete_decision_instance(decision_evaluation_key, delete_decision_instance_request)
Delete decision instance

Delete all associated decision evaluations based on provided key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_evaluation_key** | **DecisionEvaluationKey** | The key of the decision evaluation to delete. | [required] |
**delete_decision_instance_request** | Option<[**DeleteDecisionInstanceRequest**](DeleteDecisionInstanceRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_decision_instances_batch_operation

> models::BatchOperationCreatedResult delete_decision_instances_batch_operation(decision_instance_deletion_batch_operation_request)
Delete decision instances (batch)

Delete multiple decision instances. This will delete the historic data from secondary storage. This is done asynchronously, the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_instance_deletion_batch_operation_request** | [**DecisionInstanceDeletionBatchOperationRequest**](DecisionInstanceDeletionBatchOperationRequest.md) |  | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_decision_instance

> models::DecisionInstanceGetQueryResult get_decision_instance(decision_evaluation_instance_key)
Get decision instance

Returns a decision instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_evaluation_instance_key** | **String** | The assigned key of the decision instance, which acts as a unique identifier for this decision instance. | [required] |

### Return type

[**models::DecisionInstanceGetQueryResult**](DecisionInstanceGetQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_decision_instances

> models::DecisionInstanceSearchQueryResult search_decision_instances(decision_instance_search_query)
Search decision instances

Search for decision instances based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_instance_search_query** | Option<[**DecisionInstanceSearchQuery**](DecisionInstanceSearchQuery.md)> |  |  |

### Return type

[**models::DecisionInstanceSearchQueryResult**](DecisionInstanceSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

