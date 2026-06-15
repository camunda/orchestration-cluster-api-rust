# DeleteResourceRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]
**delete_history** | Option<**bool**> | Indicates if the historic data of a process resource should be deleted via a batch operation asynchronously.  This flag is only effective for process resources. For other resource types (decisions, forms, generic resources), this flag is ignored and no history will be deleted. In those cases, the `batchOperation` field in the response will not be populated.  | [optional][default to false]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


