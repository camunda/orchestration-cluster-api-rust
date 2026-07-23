# AdvancedMetadataValueFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dollar_eq** | Option<[**models::CreateClusterVariableRequestMetadataValue**](CreateClusterVariableRequestMetadataValue.md)> |  | [optional]
**dollar_neq** | Option<[**models::CreateClusterVariableRequestMetadataValue**](CreateClusterVariableRequestMetadataValue.md)> |  | [optional]
**dollar_exists** | Option<**bool**> | Checks if the metadata key exists. | [optional]
**dollar_gt** | Option<**f64**> | Greater than comparison with the provided value. | [optional]
**dollar_gte** | Option<**f64**> | Greater than or equal comparison with the provided value. | [optional]
**dollar_lt** | Option<**f64**> | Lower than comparison with the provided value. | [optional]
**dollar_lte** | Option<**f64**> | Lower than or equal comparison with the provided value. | [optional]
**dollar_in** | Option<[**Vec<models::CreateClusterVariableRequestMetadataValue>**](CreateClusterVariableRequestMetadataValue.md)> | Checks if the property matches any of the provided values. | [optional]
**dollar_like** | Option<**String**> | Checks if the property matches the provided like value.  Supported wildcard characters are:  * `*`: matches zero, one, or multiple characters. * `?`: matches one, single character.  Wildcard characters can be escaped with backslash, for instance: `\\*`.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


