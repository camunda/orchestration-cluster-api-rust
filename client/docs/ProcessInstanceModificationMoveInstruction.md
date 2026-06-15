# ProcessInstanceModificationMoveInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**source_element_instruction** | [**models::SourceElementInstruction**](SourceElementInstruction.md) |  | 
**target_element_id** | **String** | The target element id. | 
**ancestor_scope_instruction** | Option<[**models::AncestorScopeInstruction**](AncestorScopeInstruction.md)> |  | [optional]
**variable_instructions** | Option<[**Vec<models::ModifyProcessInstanceVariableInstruction>**](ModifyProcessInstanceVariableInstruction.md)> | Instructions describing which variables to create or update. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


