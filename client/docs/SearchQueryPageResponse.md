# SearchQueryPageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**total_items** | **i64** | Total items matching the criteria. | 
**has_more_total_items** | **bool** | Indicates whether the `totalItems` value has been capped due to system limits. When true, `totalItems` is a lower bound and the actual number of matching items is greater than the reported value.  | 
**start_cursor** | Option<**String**> | The cursor value for getting the previous page of results. Use this in the `before` field of an ensuing request. | 
**end_cursor** | Option<**String**> | The cursor value for getting the next page of results. Use this in the `after` field of an ensuing request. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


