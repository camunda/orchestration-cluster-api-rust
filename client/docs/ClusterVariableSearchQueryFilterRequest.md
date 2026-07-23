# ClusterVariableSearchQueryFilterRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Name of the cluster variable. | [optional]
**value** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The value of the cluster variable. | [optional]
**scope** | Option<[**models::ClusterVariableScopeFilterProperty**](ClusterVariableScopeFilterProperty.md)> | The scope filter for cluster variables. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Tenant ID of this variable. | [optional]
**is_truncated** | Option<**bool**> | Filter cluster variables by truncation status of their stored values. When true, returns only variables whose stored values are truncated (i.e., the value exceeds the storage size limit and is truncated in storage). When false, returns only variables with non-truncated stored values. This filter is based on the underlying storage characteristic, not the response format.  | [optional]
**metadata** | Option<[**std::collections::HashMap<String, models::AdvancedMetadataValueFilter>**](AdvancedMetadataValueFilter.md)> | Filter by metadata entries. A map of metadata key to an advanced filter on that key's value. Metadata values are strings or numbers. | [optional]
**kind** | Option<[**models::ClusterVariableKindFilterProperty**](ClusterVariableKindFilterProperty.md)> | The kind filter for cluster variables. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


