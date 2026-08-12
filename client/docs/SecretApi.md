# \SecretApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_secrets**](SecretApi.md#list_secrets) | **POST** /secrets/list | List secrets (alpha)
[**resolve_secrets**](SecretApi.md#resolve_secrets) | **POST** /secrets/resolve | Resolve secrets (alpha)



## list_secrets

> models::SecretListResult list_secrets(body)
List secrets (alpha)

List the `camunda.secrets.*` references known for the caller's physical tenant.  Only references the caller holds `SECRET:READ` on are returned. This endpoint never returns secret values, only the reference names.  The references are read from the secret stores configured for the caller's physical tenant. Secret names that cannot form a valid `camunda.secrets.<name>` reference (for example names containing a dot or a dash) are omitted, since they could neither be resolved nor be used in a BPMN expression.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | Option<**serde_json::Value**> |  |  |

### Return type

[**models::SecretListResult**](SecretListResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resolve_secrets

> models::SecretResolveResult resolve_secrets(secret_resolve_request)
Resolve secrets (alpha)

Resolve a deduplicated batch of `camunda.secrets.*` references for the caller's physical tenant in a single round-trip.  Each reference is authorized and resolved independently. For valid requests, the endpoint always responds with HTTP 200: successfully resolved references are returned in `resolved`, while references that could not be resolved (for example not found, malformed or over-long, or the caller lacks `SECRET:REVEAL` on that reference) are returned in `errors`. A failure of one reference never fails the others. Only structurally invalid requests are rejected with HTTP 400: a missing or non-array `references` field, more than 20 references, or a null entry.  References are resolved against the secret stores configured for the caller's physical tenant, served from the gateway's secret cache when the value is already cached and read from the store otherwise.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**secret_resolve_request** | [**SecretResolveRequest**](SecretResolveRequest.md) |  | [required] |

### Return type

[**models::SecretResolveResult**](SecretResolveResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

