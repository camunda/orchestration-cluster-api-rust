# DeleteResourceRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]
**delete_history** | Option<**bool**> | Indicates if the historic data associated with the resource should also be deleted asynchronously.  This flag is effective for process definitions and decision requirements definitions. For other resource types (forms, generic resources) it is ignored and no history is deleted. For a decision requirements definition the `batchOperation` field in the response carries the created batch operation. For a process definition the history is deleted as part of the definition's draining/deletion lifecycle and no batch operation is returned.  | [optional][default to false]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


