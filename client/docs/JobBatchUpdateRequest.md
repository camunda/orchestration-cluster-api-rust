# JobBatchUpdateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**models::JobFilter**](JobFilter.md) | The job filter. At least one dimension must be set. | 
**changeset** | [**models::JobChangeset**](JobChangeset.md) | The fields to update. At least one field must be non-null. | 
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


