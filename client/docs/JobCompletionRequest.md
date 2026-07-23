# JobCompletionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The variables to complete the job with. | [optional]
**result** | Option<[**models::JobResult**](JobResult.md)> |  | [optional]
**lease_token** | Option<**String**> | The token identifying a leased job's activation, obtained from `ActivatedJobResult.leaseToken`. For a leased job, the matching token must be supplied to prove the command comes from the worker that holds the current lease; a command with no token is rejected. A command carrying a stale token is likewise rejected, fencing the job against a superseded activation (for example, after the job timed out or failed and was re-activated by another worker). A job that was activated without a lease requires no token.  | [optional]
**business_id** | Option<**String**> | An optional business id to assign to the process instance the job belongs to, as part of completing the job, letting a worker set the identifier from work it just performed. The business id can only be assigned to a root process instance: if the job belongs to a child process instance (one started by a call activity), the completion is rejected. An empty business id is likewise rejected. The assignment is single and irreversible and is only accepted while business id uniqueness is disabled. Only artifacts created after the assignment carry the business id; already-existing ones are not enriched. Completing with a business id that differs from one already assigned rejects the whole completion, leaving the job open; re-sending the identical business id is an idempotent no-op.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


