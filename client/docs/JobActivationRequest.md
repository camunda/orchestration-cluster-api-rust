# JobActivationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** | The job type, as defined in the BPMN process (e.g. <zeebe:taskDefinition type=\"payment-service\" />) | 
**worker** | Option<**String**> | The name of the worker activating the jobs, mostly used for logging purposes. | [optional]
**timeout** | **i64** | A job returned after this call will not be activated by another call until the timeout (in ms) has been reached.  | 
**max_jobs_to_activate** | **i32** | The maximum jobs to activate by this request. | 
**fetch_variable** | Option<**Vec<String>**> | A list of variables to fetch as the job variables; if empty, all visible variables at the time of activation for the scope of the job will be returned. | [optional]
**request_timeout** | Option<**i64**> | The request will be completed when at least one job is activated or after the requestTimeout (in ms). If the requestTimeout = 0, a default timeout is used. If the requestTimeout < 0, long polling is disabled and the request is completed immediately, even when no job is activated.  | [optional]
**tenant_ids** | Option<**Vec<String>**> | A list of IDs of tenants for which to activate jobs. | [optional]
**tenant_filter** | Option<[**models::TenantFilterEnum**](TenantFilterEnum.md)> | The tenant filtering strategy - determines whether to use provided tenant IDs or assigned tenant IDs from the authenticated principal's authorized tenants.  | [optional][default to Provided]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


