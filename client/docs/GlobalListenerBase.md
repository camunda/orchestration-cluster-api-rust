# GlobalListenerBase

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | Option<**String**> | The name of the job type, used as a reference to specify which job workers request the respective listener job. | [optional]
**retries** | Option<**i32**> | Number of retries for the listener job. | [optional]
**after_non_global** | Option<**bool**> | Whether the listener should run after model-level listeners. | [optional]
**priority** | Option<**i32**> | The priority of the listener. Higher priority listeners are executed before lower priority ones. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


