# AdvancedProtectionModeFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dollar_eq** | Option<[**models::ProtectionModeEnum**](ProtectionModeEnum.md)> | Checks if the variable has the provided protection mode. | [optional]
**dollar_neq** | Option<[**models::ProtectionModeEnum**](ProtectionModeEnum.md)> | Checks if the variable does not have the provided protection mode. | [optional]
**dollar_exists** | Option<**bool**> | Checks if the variable has any protection mode. Use `false` to match the variables that are not protected.  | [optional]
**dollar_in** | Option<[**Vec<models::ProtectionModeEnum>**](ProtectionModeEnum.md)> | Checks if the variable has at least one of the provided protection modes. | [optional]
**dollar_not_in** | Option<[**Vec<models::ProtectionModeEnum>**](ProtectionModeEnum.md)> | Checks if the variable has none of the provided protection modes. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


