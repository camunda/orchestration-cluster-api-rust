# ProcessDefinitionResumptionInstructionById

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | The BPMN process ID of the process definition. | 
**process_definition_version** | Option<**i32**> | The version of the process definition. Must be 1 or greater; `-1` is not accepted. If omitted, all versions are affected.  | [optional]
**tenant_id** | Option<**String**> | The tenant of the process definition. Required when multi-tenancy is enabled; if multi-tenancy is disabled, it can be omitted and the default tenant is used.  | [optional]
**include_process_instances** | Option<**bool**> | Whether to also resume the process instances of the affected process definition versions. The instances are changed by a batch operation.  | [optional][default to false]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


