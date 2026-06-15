# JobWaitStateDetails

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**wait_state_type** | **String** | The wait state type discriminator. | 
**job_key** | **models::JobKey** | The key of the job. | 
**job_type** | **String** | The job type (worker subscription identifier). | 
**job_kind** | [**models::JobKindEnum**](JobKindEnum.md) | The kind of job. | 
**listener_event_type** | Option<[**models::JobListenerEventTypeEnum**](JobListenerEventTypeEnum.md)> | The listener event type of the job (only set for execution listener and task listener jobs). | 
**retries** | Option<**i32**> | The number of retries remaining for the job. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


