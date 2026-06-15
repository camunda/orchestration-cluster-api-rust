# JobErrorRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_code** | **String** | The error code that will be matched with an error catch event.  | 
**error_message** | Option<**String**> | An error message that provides additional context.  | [optional]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | JSON object that will instantiate the variables at the local scope of the error catch event that catches the thrown error.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


