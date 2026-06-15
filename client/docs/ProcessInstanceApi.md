# \ProcessInstanceApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_process_instance**](ProcessInstanceApi.md#cancel_process_instance) | **POST** /process-instances/{processInstanceKey}/cancellation | Cancel process instance
[**cancel_process_instances_batch_operation**](ProcessInstanceApi.md#cancel_process_instances_batch_operation) | **POST** /process-instances/cancellation | Cancel process instances (batch)
[**create_process_instance**](ProcessInstanceApi.md#create_process_instance) | **POST** /process-instances | Create process instance
[**delete_process_instance**](ProcessInstanceApi.md#delete_process_instance) | **POST** /process-instances/{processInstanceKey}/deletion | Delete process instance
[**delete_process_instances_batch_operation**](ProcessInstanceApi.md#delete_process_instances_batch_operation) | **POST** /process-instances/deletion | Delete process instances (batch)
[**get_process_instance**](ProcessInstanceApi.md#get_process_instance) | **GET** /process-instances/{processInstanceKey} | Get process instance
[**get_process_instance_call_hierarchy**](ProcessInstanceApi.md#get_process_instance_call_hierarchy) | **GET** /process-instances/{processInstanceKey}/call-hierarchy | Get call hierarchy
[**get_process_instance_sequence_flows**](ProcessInstanceApi.md#get_process_instance_sequence_flows) | **GET** /process-instances/{processInstanceKey}/sequence-flows | Get sequence flows
[**get_process_instance_statistics**](ProcessInstanceApi.md#get_process_instance_statistics) | **GET** /process-instances/{processInstanceKey}/statistics/element-instances | Get element instance statistics
[**migrate_process_instance**](ProcessInstanceApi.md#migrate_process_instance) | **POST** /process-instances/{processInstanceKey}/migration | Migrate process instance
[**migrate_process_instances_batch_operation**](ProcessInstanceApi.md#migrate_process_instances_batch_operation) | **POST** /process-instances/migration | Migrate process instances (batch)
[**modify_process_instance**](ProcessInstanceApi.md#modify_process_instance) | **POST** /process-instances/{processInstanceKey}/modification | Modify process instance
[**modify_process_instances_batch_operation**](ProcessInstanceApi.md#modify_process_instances_batch_operation) | **POST** /process-instances/modification | Modify process instances (batch)
[**resolve_incidents_batch_operation**](ProcessInstanceApi.md#resolve_incidents_batch_operation) | **POST** /process-instances/incident-resolution | Resolve related incidents (batch)
[**resolve_process_instance_incidents**](ProcessInstanceApi.md#resolve_process_instance_incidents) | **POST** /process-instances/{processInstanceKey}/incident-resolution | Resolve related incidents
[**search_process_instance_incidents**](ProcessInstanceApi.md#search_process_instance_incidents) | **POST** /process-instances/{processInstanceKey}/incidents/search | Search related incidents
[**search_process_instances**](ProcessInstanceApi.md#search_process_instances) | **POST** /process-instances/search | Search process instances



## cancel_process_instance

> cancel_process_instance(process_instance_key, cancel_process_instance_request)
Cancel process instance

Cancels a running process instance. As a cancellation includes more than just the removal of the process instance resource, the cancellation resource must be posted. Cancellation can wait on listener-related processing; when that processing does not complete in time, this endpoint can return 504. Other gateway timeout causes are also possible. Retry with backoff and inspect listener worker availability and logs when this repeats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The key of the process instance to cancel. | [required] |
**cancel_process_instance_request** | Option<[**CancelProcessInstanceRequest**](CancelProcessInstanceRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## cancel_process_instances_batch_operation

> models::BatchOperationCreatedResult cancel_process_instances_batch_operation(process_instance_cancellation_batch_operation_request)
Cancel process instances (batch)

Cancels multiple running process instances. Since only ACTIVE root instances can be cancelled, any given filters for state and parentProcessInstanceKey are ignored and overridden during this batch operation. This is done asynchronously, the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_cancellation_batch_operation_request** | [**ProcessInstanceCancellationBatchOperationRequest**](ProcessInstanceCancellationBatchOperationRequest.md) |  | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_process_instance

> models::CreateProcessInstanceResult create_process_instance(process_instance_creation_instruction)
Create process instance

Creates and starts an instance of the specified process. The process definition to use to create the instance can be specified either using its unique key (as returned by Deploy resources), or using the BPMN process id and a version.  Waits for the completion of the process instance before returning a result when awaitCompletion is enabled. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_creation_instruction** | [**ProcessInstanceCreationInstruction**](ProcessInstanceCreationInstruction.md) |  | [required] |

### Return type

[**models::CreateProcessInstanceResult**](CreateProcessInstanceResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_process_instance

> delete_process_instance(process_instance_key, delete_process_instance_request)
Delete process instance

Deletes a process instance. Only instances that are completed or terminated can be deleted.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The key of the process instance to delete. | [required] |
**delete_process_instance_request** | Option<[**DeleteProcessInstanceRequest**](DeleteProcessInstanceRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_process_instances_batch_operation

> models::BatchOperationCreatedResult delete_process_instances_batch_operation(process_instance_deletion_batch_operation_request)
Delete process instances (batch)

Delete multiple process instances. This will delete the historic data from secondary storage. Only process instances in a final state (COMPLETED or TERMINATED) can be deleted. This is done asynchronously, the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_deletion_batch_operation_request** | [**ProcessInstanceDeletionBatchOperationRequest**](ProcessInstanceDeletionBatchOperationRequest.md) |  | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_instance

> models::ProcessInstanceResult get_process_instance(process_instance_key)
Get process instance

Get the process instance by the process instance key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The process instance key. | [required] |

### Return type

[**models::ProcessInstanceResult**](ProcessInstanceResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_instance_call_hierarchy

> Vec<models::ProcessInstanceCallHierarchyEntry> get_process_instance_call_hierarchy(process_instance_key)
Get call hierarchy

Returns the call hierarchy for a given process instance, showing its ancestry up to the root instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The key of the process instance to fetch the hierarchy for. | [required] |

### Return type

[**Vec<models::ProcessInstanceCallHierarchyEntry>**](ProcessInstanceCallHierarchyEntry.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_instance_sequence_flows

> models::ProcessInstanceSequenceFlowsQueryResult get_process_instance_sequence_flows(process_instance_key)
Get sequence flows

Get sequence flows taken by the process instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The assigned key of the process instance, which acts as a unique identifier for this process instance. | [required] |

### Return type

[**models::ProcessInstanceSequenceFlowsQueryResult**](ProcessInstanceSequenceFlowsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_process_instance_statistics

> models::ProcessInstanceElementStatisticsQueryResult get_process_instance_statistics(process_instance_key)
Get element instance statistics

Get statistics about elements by the process instance key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The assigned key of the process instance, which acts as a unique identifier for this process instance. | [required] |

### Return type

[**models::ProcessInstanceElementStatisticsQueryResult**](ProcessInstanceElementStatisticsQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## migrate_process_instance

> migrate_process_instance(process_instance_key, process_instance_migration_instruction)
Migrate process instance

Migrates a process instance to a new process definition. This request can contain multiple mapping instructions to define mapping between the active process instance's elements and target process definition elements.  Use this to upgrade a process instance to a new version of a process or to a different process definition, e.g. to keep your running instances up-to-date with the latest process improvements. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The key of the process instance that should be migrated. | [required] |
**process_instance_migration_instruction** | [**ProcessInstanceMigrationInstruction**](ProcessInstanceMigrationInstruction.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## migrate_process_instances_batch_operation

> models::BatchOperationCreatedResult migrate_process_instances_batch_operation(process_instance_migration_batch_operation_request)
Migrate process instances (batch)

Migrate multiple process instances. Since only process instances with ACTIVE state can be migrated, any given filters for state are ignored and overridden during this batch operation. This is done asynchronously, the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_migration_batch_operation_request** | [**ProcessInstanceMigrationBatchOperationRequest**](ProcessInstanceMigrationBatchOperationRequest.md) |  | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_process_instance

> modify_process_instance(process_instance_key, process_instance_modification_instruction)
Modify process instance

Modifies a running process instance. This request can contain multiple instructions to activate an element of the process or to terminate an active instance of an element.  Use this to repair a process instance that is stuck on an element or took an unintended path. For example, because an external system is not available or doesn't respond as expected. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The key of the process instance that should be modified. | [required] |
**process_instance_modification_instruction** | [**ProcessInstanceModificationInstruction**](ProcessInstanceModificationInstruction.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_process_instances_batch_operation

> models::BatchOperationCreatedResult modify_process_instances_batch_operation(process_instance_modification_batch_operation_request)
Modify process instances (batch)

Modify multiple process instances. Since only process instances with ACTIVE state can be modified, any given filters for state are ignored and overridden during this batch operation. In contrast to single modification operation, it is not possible to add variable instructions or modify by element key. It is only possible to use the element id of the source and target. This is done asynchronously, the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_modification_batch_operation_request** | [**ProcessInstanceModificationBatchOperationRequest**](ProcessInstanceModificationBatchOperationRequest.md) |  | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resolve_incidents_batch_operation

> models::BatchOperationCreatedResult resolve_incidents_batch_operation(process_instance_incident_resolution_batch_operation_request)
Resolve related incidents (batch)

Resolves multiple instances of process instances. Since only process instances with ACTIVE state can have unresolved incidents, any given filters for state are ignored and overridden during this batch operation. This is done asynchronously, the progress can be tracked using the batchOperationKey from the response and the batch operation status endpoint (/batch-operations/{batchOperationKey}). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_incident_resolution_batch_operation_request** | Option<[**ProcessInstanceIncidentResolutionBatchOperationRequest**](ProcessInstanceIncidentResolutionBatchOperationRequest.md)> |  |  |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resolve_process_instance_incidents

> models::BatchOperationCreatedResult resolve_process_instance_incidents(process_instance_key)
Resolve related incidents

Creates a batch operation to resolve multiple incidents of a process instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The key of the process instance to resolve incidents for. | [required] |

### Return type

[**models::BatchOperationCreatedResult**](BatchOperationCreatedResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_process_instance_incidents

> models::IncidentSearchQueryResult search_process_instance_incidents(process_instance_key, incident_search_query)
Search related incidents

Search for incidents caused by the process instance or any of its called process or decision instances.  Although the `processInstanceKey` is provided as a path parameter to indicate the root process instance, you may also include a `processInstanceKey` within the filter object to narrow results to specific child process instances. This is useful, for example, if you want to isolate incidents associated with subprocesses or called processes under the root instance while excluding incidents directly tied to the root. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_key** | **ProcessInstanceKey** | The assigned key of the process instance, which acts as a unique identifier for this process instance. | [required] |
**incident_search_query** | Option<[**IncidentSearchQuery**](IncidentSearchQuery.md)> |  |  |

### Return type

[**models::IncidentSearchQueryResult**](IncidentSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_process_instances

> models::ProcessInstanceSearchQueryResult search_process_instances(process_instance_search_query)
Search process instances

Search for process instances based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**process_instance_search_query** | Option<[**ProcessInstanceSearchQuery**](ProcessInstanceSearchQuery.md)> |  |  |

### Return type

[**models::ProcessInstanceSearchQueryResult**](ProcessInstanceSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

