# ProcessInstanceModificationInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]
**activate_instructions** | Option<[**Vec<models::ProcessInstanceModificationActivateInstruction>**](ProcessInstanceModificationActivateInstruction.md)> | Instructions describing which elements to activate in which scopes and which variables to create or update. | [optional]
**move_instructions** | Option<[**Vec<models::ProcessInstanceModificationMoveInstruction>**](ProcessInstanceModificationMoveInstruction.md)> | Instructions describing which elements to move from one scope to another. | [optional]
**terminate_instructions** | Option<[**Vec<models::ProcessInstanceModificationTerminateInstruction>**](ProcessInstanceModificationTerminateInstruction.md)> | Instructions describing which elements to terminate. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


