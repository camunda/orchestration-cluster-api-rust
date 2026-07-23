# JobUpdateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**changeset** | [**models::JobChangeset**](JobChangeset.md) |  | 
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]
**lease_token** | Option<**String**> | The token identifying a leased job's activation, obtained from `ActivatedJobResult.leaseToken`. For a leased job, a supplied token is validated to prove the command comes from the worker that holds the current lease; a command carrying a stale token is rejected, fencing the job against a superseded activation (for example, after the job timed out or failed and was re-activated by another worker). An update without a token always applies to support operator and bulk updates of leased jobs. Note that this is different from lifecycle requests like complete, fail, and throw-error that always require a token for leased jobs. A job that was activated without a lease requires no token.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


