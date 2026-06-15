# \MessageApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**correlate_message**](MessageApi.md#correlate_message) | **POST** /messages/correlation | Correlate message
[**publish_message**](MessageApi.md#publish_message) | **POST** /messages/publication | Publish message



## correlate_message

> models::MessageCorrelationResult correlate_message(message_correlation_request)
Correlate message

Publishes a message and correlates it to a subscription. If correlation is successful it will return the first process instance key the message correlated with. The message is not buffered. Use the publish message endpoint to send messages that can be buffered. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**message_correlation_request** | [**MessageCorrelationRequest**](MessageCorrelationRequest.md) |  | [required] |

### Return type

[**models::MessageCorrelationResult**](MessageCorrelationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## publish_message

> models::MessagePublicationResult publish_message(message_publication_request)
Publish message

Publishes a single message. Messages are published to specific partitions computed from their correlation keys. Messages can be buffered. The endpoint does not wait for a correlation result. Use the message correlation endpoint for such use cases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**message_publication_request** | [**MessagePublicationRequest**](MessagePublicationRequest.md) |  | [required] |

### Return type

[**models::MessagePublicationResult**](MessagePublicationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

