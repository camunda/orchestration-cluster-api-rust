# ProbeCreationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**probe_id** | **String** | The probe ID, as the worker registered it: 1 to 256 characters, without whitespace, and not `*`. The engine matches it to the probe IDs of polling workers by exact equality.  | 
**input** | **String** | A FEEL expression that evaluates to a context: the question for the worker, whose shape the worker of the probe ID defines. References to `camunda.vars` and `camunda.secrets` follow the rules of input mappings.  | 
**tenant_id** | Option<**String**> | The tenant of the probe. Required with multi-tenancy. | [optional]
**request_timeout** | Option<**i64**> | How long to wait for an answer, in milliseconds, from 0 to 30000. With `0` or absent, the cluster's request timeout is used, capped at 30000.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


