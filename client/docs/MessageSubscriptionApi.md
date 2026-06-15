# \MessageSubscriptionApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_correlated_message_subscriptions**](MessageSubscriptionApi.md#search_correlated_message_subscriptions) | **POST** /correlated-message-subscriptions/search | Search correlated message subscriptions
[**search_message_subscriptions**](MessageSubscriptionApi.md#search_message_subscriptions) | **POST** /message-subscriptions/search | Search message subscriptions



## search_correlated_message_subscriptions

> models::CorrelatedMessageSubscriptionSearchQueryResult search_correlated_message_subscriptions(correlated_message_subscription_search_query)
Search correlated message subscriptions

Search correlated message subscriptions based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**correlated_message_subscription_search_query** | Option<[**CorrelatedMessageSubscriptionSearchQuery**](CorrelatedMessageSubscriptionSearchQuery.md)> |  |  |

### Return type

[**models::CorrelatedMessageSubscriptionSearchQueryResult**](CorrelatedMessageSubscriptionSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_message_subscriptions

> models::MessageSubscriptionSearchQueryResult search_message_subscriptions(message_subscription_search_query)
Search message subscriptions

Search for message subscriptions based on given criteria.  By default, both start and intermediate event subscriptions are returned. Use the `messageSubscriptionType` filter to restrict results to a single type.  **Version notes:** - Start event subscriptions are only captured for deployments made with 8.10 or later. - The `messageSubscriptionType` field is only populated for data created   with Camunda 8.10 or later. For pre-8.10 data, intermediate event entries have no   `messageSubscriptionType` value stored. For convenience, the API returns `PROCESS_EVENT`   as a default for such search results, though. - Searching for intermediate event subscriptions **including legacy data** can be achieved   by filtering for `messageSubscriptionType` not matching `START_EVENT`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**message_subscription_search_query** | Option<[**MessageSubscriptionSearchQuery**](MessageSubscriptionSearchQuery.md)> |  |  |

### Return type

[**models::MessageSubscriptionSearchQueryResult**](MessageSubscriptionSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

