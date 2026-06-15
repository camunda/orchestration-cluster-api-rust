# BatchOperationResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**batch_operation_key** | **String** | Key or (Operate Legacy ID = UUID) of the batch operation. | 
**state** | [**models::BatchOperationStateEnum**](BatchOperationStateEnum.md) |  | 
**batch_operation_type** | [**models::BatchOperationTypeEnum**](BatchOperationTypeEnum.md) |  | 
**start_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The start date of the batch operation. This is `null` if the batch operation has not yet started.  | 
**end_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The end date of the batch operation. This is `null` if the batch operation is still running.  | 
**actor_type** | Option<[**models::AuditLogActorTypeEnum**](AuditLogActorTypeEnum.md)> | The type of the actor who performed the operation. This is `null` if the batch operation was created before 8.9, or if the actor information is not available.  | 
**actor_id** | Option<**String**> | The ID of the actor who performed the operation. Available for batch operations created since 8.9. | 
**operations_total_count** | **i32** | The total number of items contained in this batch operation. | 
**operations_failed_count** | **i32** | The number of items which failed during execution of the batch operation. (e.g. because they are rejected by the Zeebe engine). | 
**operations_completed_count** | **i32** | The number of successfully completed tasks. | 
**errors** | [**Vec<models::BatchOperationError>**](BatchOperationError.md) | The errors that occurred per partition during the batch operation. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


