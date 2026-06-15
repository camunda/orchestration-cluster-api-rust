# BatchOperationFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**batch_operation_key** | Option<[**models::BasicStringFilterProperty**](BasicStringFilterProperty.md)> | The key (or operate legacy ID) of the batch operation. | [optional]
**operation_type** | Option<[**models::BatchOperationTypeFilterProperty**](BatchOperationTypeFilterProperty.md)> | The type of the batch operation. | [optional]
**state** | Option<[**models::BatchOperationStateFilterProperty**](BatchOperationStateFilterProperty.md)> | The state of the batch operation. | [optional]
**actor_type** | Option<[**models::AuditLogActorTypeEnum**](AuditLogActorTypeEnum.md)> | The type of the actor who performed the operation. | [optional]
**actor_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The ID of the actor who performed the operation. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


