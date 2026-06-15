# JobErrorStatisticsFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from** | **chrono::DateTime<chrono::FixedOffset>** | Start of the time window to filter metrics. ISO 8601 date-time format.  | 
**to** | **chrono::DateTime<chrono::FixedOffset>** | End of the time window to filter metrics. ISO 8601 date-time format.  | 
**job_type** | **String** | Job type to return error metrics for. | 
**error_code** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Optional error code filter with advanced search capabilities. | [optional]
**error_message** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Optional error message filter with advanced search capabilities. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


