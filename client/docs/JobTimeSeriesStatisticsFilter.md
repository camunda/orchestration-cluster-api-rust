# JobTimeSeriesStatisticsFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from** | **chrono::DateTime<chrono::FixedOffset>** | Start of the time window to filter metrics. ISO 8601 date-time format.  | 
**to** | **chrono::DateTime<chrono::FixedOffset>** | End of the time window to filter metrics. ISO 8601 date-time format.  | 
**job_type** | **String** | Job type to return time-series metrics for. | 
**resolution** | Option<**String**> | Time bucket resolution as an ISO 8601 duration (for example `PT1M` for 1 minute, `PT1H` for 1 hour). If omitted, the server chooses a sensible default.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


