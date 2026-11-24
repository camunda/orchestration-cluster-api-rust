# \SecretApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_secrets**](SecretApi.md#list_secrets) | **POST** /secrets/list | List secrets
[**resolve_secrets**](SecretApi.md#resolve_secrets) | **POST** /secrets/resolve | Resolve secrets



## list_secrets

> models::SecretListResult list_secrets(secret_list_request)
List secrets

List the `camunda.secrets.*` references known for the caller's physical tenant.  Only references the caller holds `SECRET:READ` on are returned. This endpoint never returns secret values, only the reference names.  The references are read from the secret stores configured for the caller's physical tenant. A store may hold names outside the reference name charset (for example one containing a dot); those are omitted, since `/secrets/resolve` would reject them and no permission can be granted on them.  A returned reference is usable verbatim with `/secrets/resolve`, although a secret that its store does not report as `ACTIVE` may fail to resolve there. In a FEEL expression, however, a name that is not a bare identifier has to be backtick-escaped, since FEEL reads a bare dash as the minus operator: a listed `camunda.secrets.db-password` is written `` =camunda.secrets.`db-password` `` in a BPMN input mapping.  Pass `filter.status` to list only the secrets in the matching states, either as a single status or with the advanced operators (`$in`, `$neq`, ...). A store that does not report states reports every secret as `ACTIVE`, so a filter that excludes `ACTIVE` returns nothing from it. The status filter does not widen access: a reference the caller may not read is never returned, with or without a filter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**secret_list_request** | Option<[**SecretListRequest**](SecretListRequest.md)> |  |  |

### Return type

[**models::SecretListResult**](SecretListResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resolve_secrets

> models::SecretResolveResult resolve_secrets(secret_resolve_request)
Resolve secrets

Resolve a deduplicated batch of `camunda.secrets.*` references for the caller's physical tenant in a single round-trip.  Each reference is authorized and resolved independently. For valid requests, the endpoint always responds with HTTP 200: successfully resolved references are returned in `resolved`, while references that could not be resolved (for example not found, malformed or over-long, or the caller lacks `SECRET:REVEAL` on that reference) are returned in `errors`. A failure of one reference never fails the others. Only structurally invalid requests are rejected with HTTP 400: a missing or non-array `references` field, more than 20 references, or a null entry.  References are resolved against the secret stores configured for the caller's physical tenant, served from the gateway's secret cache when the value is already cached and read from the store otherwise. 

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
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

