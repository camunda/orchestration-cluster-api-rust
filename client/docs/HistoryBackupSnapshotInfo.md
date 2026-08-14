# HistoryBackupSnapshotInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**snapshot_name** | **String** | The name of the snapshot. | [readonly]
**state** | Option<**String**> | The state of the snapshot, reported verbatim by the secondary storage (for example 'SUCCESS', 'IN_PROGRESS' or 'PARTIAL'). Deliberately not a closed set: Elasticsearch and OpenSearch report different vocabularies. Not reported when the backup was listed without snapshot detail.  | [readonly]
**start_time** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The timestamp at which the snapshot was started. Not reported when the backup was listed without snapshot detail.  | [readonly]
**failures** | **Vec<String>** | The failures reported for this snapshot. Empty if there were none. | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


