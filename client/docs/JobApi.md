# \JobApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**activate_jobs**](JobApi.md#activate_jobs) | **POST** /jobs/activation | Activate jobs
[**complete_job**](JobApi.md#complete_job) | **POST** /jobs/{jobKey}/completion | Complete job
[**fail_job**](JobApi.md#fail_job) | **POST** /jobs/{jobKey}/failure | Fail job
[**get_global_job_statistics**](JobApi.md#get_global_job_statistics) | **GET** /jobs/statistics/global | Global job statistics
[**get_job_error_statistics**](JobApi.md#get_job_error_statistics) | **POST** /jobs/statistics/errors | Get error metrics for a job type
[**get_job_time_series_statistics**](JobApi.md#get_job_time_series_statistics) | **POST** /jobs/statistics/time-series | Get time-series metrics for a job type
[**get_job_type_statistics**](JobApi.md#get_job_type_statistics) | **POST** /jobs/statistics/by-types | Get job statistics by type
[**get_job_worker_statistics**](JobApi.md#get_job_worker_statistics) | **POST** /jobs/statistics/by-workers | Get job statistics by worker
[**search_jobs**](JobApi.md#search_jobs) | **POST** /jobs/search | Search jobs
[**throw_job_error**](JobApi.md#throw_job_error) | **POST** /jobs/{jobKey}/error | Throw error for job
[**update_job**](JobApi.md#update_job) | **PATCH** /jobs/{jobKey} | Update job
[**update_jobs_batch_operation**](JobApi.md#update_jobs_batch_operation) | **POST** /jobs/batch-update | Update jobs (batch)



## activate_jobs

> models::JobActivationResult activate_jobs(job_activation_request)
Activate jobs

Iterate through all known partitions and activate jobs up to the requested maximum. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_activation_request** | [**JobActivationRequest**](JobActivationRequest.md) |  | [required] |

### Return type

[**models::JobActivationResult**](JobActivationResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## complete_job

> complete_job(job_key, job_completion_request)
Complete job

Complete a job with the given payload, which allows completing the associated service task. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_key** | **JobKey** | The key of the job to complete. | [required] |
**job_completion_request** | Option<[**JobCompletionRequest**](JobCompletionRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fail_job

> fail_job(job_key, job_fail_request)
Fail job

Mark the job as failed. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_key** | **JobKey** | The key of the job to fail. | [required] |
**job_fail_request** | Option<[**JobFailRequest**](JobFailRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_global_job_statistics

> models::GlobalJobStatisticsQueryResult get_global_job_statistics(from, to, job_type)
Global job statistics

Returns global aggregated counts for jobs. Filter by the creation time window (required) and optionally by jobType. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**from** | **chrono::DateTime<chrono::FixedOffset>** | Start of the time window to filter metrics. ISO 8601 date-time format.  | [required] |
**to** | **chrono::DateTime<chrono::FixedOffset>** | End of the time window to filter metrics. ISO 8601 date-time format.  | [required] |
**job_type** | Option<**String**> | Optional job type to limit the aggregation to a single job type. |  |

### Return type

[**models::GlobalJobStatisticsQueryResult**](GlobalJobStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_error_statistics

> models::JobErrorStatisticsQueryResult get_job_error_statistics(job_error_statistics_query)
Get error metrics for a job type

Returns aggregated metrics per error for the given jobType. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_error_statistics_query** | [**JobErrorStatisticsQuery**](JobErrorStatisticsQuery.md) |  | [required] |

### Return type

[**models::JobErrorStatisticsQueryResult**](JobErrorStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_time_series_statistics

> models::JobTimeSeriesStatisticsQueryResult get_job_time_series_statistics(job_time_series_statistics_query)
Get time-series metrics for a job type

Returns a list of time-bucketed metrics ordered ascending by time. The `from` and `to` fields select the time window of interest. Each item in the response corresponds to one time bucket of the requested resolution. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_time_series_statistics_query** | [**JobTimeSeriesStatisticsQuery**](JobTimeSeriesStatisticsQuery.md) |  | [required] |

### Return type

[**models::JobTimeSeriesStatisticsQueryResult**](JobTimeSeriesStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_type_statistics

> models::JobTypeStatisticsQueryResult get_job_type_statistics(job_type_statistics_query)
Get job statistics by type

Get statistics about jobs, grouped by job type. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_type_statistics_query** | [**JobTypeStatisticsQuery**](JobTypeStatisticsQuery.md) |  | [required] |

### Return type

[**models::JobTypeStatisticsQueryResult**](JobTypeStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_worker_statistics

> models::JobWorkerStatisticsQueryResult get_job_worker_statistics(job_worker_statistics_query)
Get job statistics by worker

Get statistics about jobs, grouped by worker, for a given job type. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_worker_statistics_query** | [**JobWorkerStatisticsQuery**](JobWorkerStatisticsQuery.md) |  | [required] |

### Return type

[**models::JobWorkerStatisticsQueryResult**](JobWorkerStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_jobs

> models::JobSearchQueryResult search_jobs(job_search_query)
Search jobs

Search for jobs based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_search_query** | Option<[**JobSearchQuery**](JobSearchQuery.md)> |  |  |

### Return type

[**models::JobSearchQueryResult**](JobSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## throw_job_error

> throw_job_error(job_key, job_error_request)
Throw error for job

Reports a business error (i.e. non-technical) that occurs while processing a job. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_key** | **JobKey** | The key of the job. | [required] |
**job_error_request** | [**JobErrorRequest**](JobErrorRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_job

> update_job(job_key, job_update_request)
Update job

Update a job with the given key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_key** | **JobKey** | The key of the job to update. | [required] |
**job_update_request** | [**JobUpdateRequest**](JobUpdateRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_jobs_batch_operation

> models::BatchOperationCreatedResult update_jobs_batch_operation(job_batch_update_request)
Update jobs (batch)

Creates a batch operation to update jobs matching the given filter. At least one changeset field must be non-null. This is done asynchronously; the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_batch_update_request** | [**JobBatchUpdateRequest**](JobBatchUpdateRequest.md) |  | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

