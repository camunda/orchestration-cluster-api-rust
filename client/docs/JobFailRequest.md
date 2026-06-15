# JobFailRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**retries** | Option<**i32**> | The amount of retries the job should have left | [optional][default to 0]
**error_message** | Option<**String**> | An optional error message describing why the job failed; if not provided, an empty string is used. | [optional]
**retry_back_off** | Option<**i64**> | An optional retry back off for the failed job. The job will not be retryable before the current time plus the back off time. The default is 0 which means the job is retryable immediately. | [optional][default to 0]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | JSON object that will instantiate the variables at the local scope of the job's associated task.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


