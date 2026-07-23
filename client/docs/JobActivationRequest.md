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
**with_lease** | Option<**bool**> | Whether to activate the jobs with a lease. When true, each activated job is assigned a distinct, opaque lease token, returned as ActivatedJobResult.leaseToken. The lease fences the complete, fail, and throw-error commands against a superseded activation of the same job (for example, after the job timed out or failed and was re-activated by another worker): a command carrying a stale lease token is rejected rather than racing with the newer activation. Once a job has been activated with a lease, it is served only to leasing workers of that job type; a homogeneous fleet per job type is recommended. Omit or set to false to activate jobs without a lease.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


