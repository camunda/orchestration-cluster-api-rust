# ProcessInstanceModificationBatchOperationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**models::ProcessInstanceFilter**](ProcessInstanceFilter.md) | The process instance filter. | 
**move_instructions** | [**Vec<models::ProcessInstanceModificationMoveBatchOperationInstruction>**](ProcessInstanceModificationMoveBatchOperationInstruction.md) | Instructions for moving tokens between elements. | 
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


