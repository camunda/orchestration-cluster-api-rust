# JobFailRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**retries** | Option<**i32**> | The amount of retries the job should have left | [optional][default to 0]
**error_message** | Option<**String**> | An optional error message describing why the job failed; if not provided, an empty string is used. | [optional]
**retry_back_off** | Option<**i64**> | An optional retry back off for the failed job. The job will not be retryable before the current time plus the back off time. The default is 0 which means the job is retryable immediately. | [optional][default to 0]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | JSON object that will instantiate the variables at the local scope of the job's associated task.  | [optional]
**lease_token** | Option<**String**> | The token identifying a leased job's activation, obtained from `ActivatedJobResult.leaseToken`. For a leased job, the matching token must be supplied to prove the command comes from the worker that holds the current lease; a command with no token is rejected. A command carrying a stale token is likewise rejected, fencing the job against a superseded activation (for example, after the job timed out or failed and was re-activated by another worker). A job that was activated without a lease requires no token.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


