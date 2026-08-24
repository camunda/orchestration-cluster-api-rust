# ClusterRestoreRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The start of the time range to restore from, as an ISO 8601 timestamp. | [optional]
**to** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The end of the time range to restore from, as an ISO 8601 timestamp. | [optional]
**backup_ids** | Option<**Vec<i64>**> | The IDs of the backups to restore from, one per partition. | [optional]
**overrides** | Option<[**std::collections::HashMap<String, models::RestoreRequest>**](RestoreRequest.md)> | The backup selection to apply to individual physical tenants, keyed by physical tenant id. Only allowed for a cluster-wide restore, that is when no `physicalTenantId` parameter is given. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


