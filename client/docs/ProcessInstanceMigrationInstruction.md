# ProcessInstanceMigrationInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**target_process_definition_key** | **models::ProcessDefinitionKey** | The key of process definition to migrate the process instance to. | 
**mapping_instructions** | [**Vec<models::MigrateProcessInstanceMappingInstruction>**](MigrateProcessInstanceMappingInstruction.md) | Element mappings from the source process instance to the target process instance. | 
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


