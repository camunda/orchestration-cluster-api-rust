# \ProbeApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**activate_probes**](ProbeApi.md#activate_probes) | **POST** /probes/activation | Activate probes (alpha)
[**answer_probe**](ProbeApi.md#answer_probe) | **POST** /probes/{probeKey}/answer | Answer probe (alpha)
[**create_probe**](ProbeApi.md#create_probe) | **POST** /probes | Create probe (alpha)



## activate_probes

> models::ProbeActivationResult activate_probes(probe_activation_request)
Activate probes (alpha)

Collects probes of the given probe IDs for a worker, up to the requested maximum.  Like job activation, the request goes to every partition. When no probe is waiting, it is held until a probe of one of these probe IDs is created or until `requestTimeout`. Probe IDs on which the caller lacks the `ANSWER` permission are skipped. The worker answers each probe with `POST /v2/probes/{probeKey}/answer` before the probe's `deadline`.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**probe_activation_request** | [**ProbeActivationRequest**](ProbeActivationRequest.md) |  | [required] |

### Return type

[**models::ProbeActivationResult**](ProbeActivationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## answer_probe

> answer_probe(probe_key, probe_answer_request)
Answer probe (alpha)

Sends a worker's answer to a probe it collected, which ends the probe.  The body is one of three shapes, chosen by `outcome`: `OK` with the result in `data`, `UNSUPPORTED`, or `ERROR` with a `message`. The client that created the probe gets this answer with the name of the worker that collected the probe.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**probe_key** | **ProbeKey** | The key of the probe to answer. | [required] |
**probe_answer_request** | [**ProbeAnswerRequest**](ProbeAnswerRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_probe

> models::ProbeResult create_probe(probe_creation_request)
Create probe (alpha)

Asks a worker a short question and waits for its answer in the same request.  The probe is routed on `probeId`: a worker that polls for this probe ID with `POST /v2/probes/activation` collects it and answers with `POST /v2/probes/{probeKey}/answer`. The engine evaluates `input` when the probe is created and injects the values of the secrets it references before a worker collects it.  The probe ends at its deadline, `requestTimeout` after the gateway received the request. Wait at least `requestTimeout` plus one second for the HTTP response, so that you get the reason the probe ended instead of a client-side timeout. With `requestTimeout` absent or `0`, the deadline uses the cluster's request timeout, capped at 30 seconds.  This endpoint is an alpha feature and may be subject to change in future releases. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**probe_creation_request** | [**ProbeCreationRequest**](ProbeCreationRequest.md) |  | [required] |

### Return type

[**models::ProbeResult**](ProbeResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

