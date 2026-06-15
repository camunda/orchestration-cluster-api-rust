# GlobalTaskListenerSearchQueryFilterRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Id of the global listener. | [optional]
**r#type** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Job type of the global listener. | [optional]
**retries** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | Number of retries of the global listener. | [optional]
**event_types** | Option<[**Vec<models::GlobalTaskListenerEventTypeFilterProperty>**](GlobalTaskListenerEventTypeFilterProperty.md)> | Event types of the global listener. | [optional]
**after_non_global** | Option<**bool**> | Whether the listener runs after model-level listeners. | [optional]
**priority** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | Priority of the global listener. | [optional]
**source** | Option<[**models::GlobalListenerSourceFilterProperty**](GlobalListenerSourceFilterProperty.md)> | How the global listener was defined. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


