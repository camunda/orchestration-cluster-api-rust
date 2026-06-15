# ProcessInstanceModificationActivateInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element_id** | **String** | The id of the element to activate. | 
**variable_instructions** | Option<[**Vec<models::ModifyProcessInstanceVariableInstruction>**](ModifyProcessInstanceVariableInstruction.md)> | Instructions describing which variables to create or update. | [optional]
**ancestor_element_instance_key** | Option<**models::ElementInstanceKey**> | The key of the ancestor scope the element instance should be created in. Set to -1 to create the new element instance within an existing element instance of the flow scope. If multiple instances of the target element's flow scope exist, choose one specifically with this property by providing its key.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


