# RestoreStatusResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **Status** | The overall status of the restore. (enum: IN_PROGRESS, COMPLETED, FAILED, CANCELLED) | 
**change_id** | **String** | The ID of the cluster change that performs the restore. | 
**started_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The time the restore started, as an ISO 8601 timestamp. | 
**brokers** | [**Vec<models::RestoreBrokerStatus>**](RestoreBrokerStatus.md) | The per-broker restore status. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


