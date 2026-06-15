# \AuditLogApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_audit_log**](AuditLogApi.md#get_audit_log) | **GET** /audit-logs/{auditLogKey} | Get audit log
[**search_audit_logs**](AuditLogApi.md#search_audit_logs) | **POST** /audit-logs/search | Search audit logs



## get_audit_log

> models::AuditLogResult get_audit_log(audit_log_key)
Get audit log

Get an audit log entry by auditLogKey.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**audit_log_key** | **AuditLogKey** | The audit log key. | [required] |

### Return type

[**models::AuditLogResult**](AuditLogResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_audit_logs

> models::AuditLogSearchQueryResult search_audit_logs(audit_log_search_query_request)
Search audit logs

Search for audit logs based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**audit_log_search_query_request** | Option<[**AuditLogSearchQueryRequest**](AuditLogSearchQueryRequest.md)> |  |  |

### Return type

[**models::AuditLogSearchQueryResult**](AuditLogSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

