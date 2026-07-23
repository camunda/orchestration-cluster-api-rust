# JobErrorRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_code** | **String** | The error code that will be matched with an error catch event.  | 
**error_message** | Option<**String**> | An error message that provides additional context.  | [optional]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | JSON object that will instantiate the variables at the local scope of the error catch event that catches the thrown error.  | [optional]
**lease_token** | Option<**String**> | The token identifying a leased job's activation, obtained from `ActivatedJobResult.leaseToken`. For a leased job, the matching token must be supplied to prove the command comes from the worker that holds the current lease; a command with no token is rejected. A command carrying a stale token is likewise rejected, fencing the job against a superseded activation (for example, after the job timed out or failed and was re-activated by another worker). A job that was activated without a lease requires no token.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


