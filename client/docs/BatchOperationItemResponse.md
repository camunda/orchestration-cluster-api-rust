# BatchOperationItemResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operation_type** | [**models::BatchOperationTypeEnum**](BatchOperationTypeEnum.md) |  | 
**batch_operation_key** | **String** | The key (or operate legacy ID) of the batch operation. | 
**item_key** | **String** | Key of the item, e.g. a process instance key. | 
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The process instance key of the processed item. Null for batch-op types whose targets are not process instances (e.g. DELETE_DECISION_INSTANCE, DELETE_DECISION_DEFINITION, DELETE_PROCESS_DEFINITION).  | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**state** | **State** | State of the item. (enum: ACTIVE, COMPLETED, SKIPPED, CANCELED, FAILED) | 
**processed_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The date this item was processed. This is `null` if the item has not yet been processed.  | 
**error_message** | Option<**String**> | The error message from the engine in case of a failed operation. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


