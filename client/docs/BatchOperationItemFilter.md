# BatchOperationItemFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**batch_operation_key** | Option<[**models::BasicStringFilterProperty**](BasicStringFilterProperty.md)> | The key (or operate legacy ID) of the batch operation. | [optional]
**item_key** | Option<[**models::BasicStringFilterProperty**](BasicStringFilterProperty.md)> | The key of the item, e.g. a process instance key. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The process instance key of the processed item. | [optional]
**state** | Option<[**models::BatchOperationItemStateFilterProperty**](BatchOperationItemStateFilterProperty.md)> | The state of the batch operation. | [optional]
**operation_type** | Option<[**models::BatchOperationTypeFilterProperty**](BatchOperationTypeFilterProperty.md)> | The type of the batch operation. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


