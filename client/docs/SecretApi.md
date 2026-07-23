# \SecretApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**resolve_secrets**](SecretApi.md#resolve_secrets) | **POST** /secrets/resolve | Resolve secrets (alpha)



## resolve_secrets

> models::SecretResolveResult resolve_secrets(secret_resolve_request)
Resolve secrets (alpha)

Resolve a deduplicated batch of `camunda.secrets.*` references for the caller's physical tenant in a single round-trip.  Each reference is authorized and resolved independently. For valid requests, the endpoint always responds with HTTP 200: successfully resolved references are returned in `resolved`, while references that could not be resolved (for example not found, malformed or over-long, or the caller lacks `SECRET:REVEAL` on that reference) are returned in `errors`. A failure of one reference never fails the others. Only structurally invalid requests are rejected with HTTP 400: a missing or non-array `references` field, more than 20 references, or a null entry.  This endpoint is an alpha feature and may be subject to change in future releases.  Phase 1: the secret backend is mocked. Only a fixed allow-list of references resolves; every other authorized, valid reference returns `NOT_FOUND`. 

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

