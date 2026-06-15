# AdvancedUserTaskStateFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dollar_eq** | Option<[**models::UserTaskStateEnum**](UserTaskStateEnum.md)> | Checks for equality with the provided value. | [optional]
**dollar_neq** | Option<[**models::UserTaskStateEnum**](UserTaskStateEnum.md)> | Checks for inequality with the provided value. | [optional]
**dollar_exists** | Option<**bool**> | Checks if the current property exists. | [optional]
**dollar_in** | Option<[**Vec<models::UserTaskStateEnum>**](UserTaskStateEnum.md)> | Checks if the property matches any of the provided values. | [optional]
**dollar_like** | Option<**String**> | Checks if the property matches the provided like value.  Supported wildcard characters are:  * `*`: matches zero, one, or multiple characters. * `?`: matches one, single character.  Wildcard characters can be escaped with backslash, for instance: `\\*`.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


