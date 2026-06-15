# AgentInstanceHistoryFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**history_item_key** | Option<[**models::AgentHistoryItemKeyFilterProperty**](AgentHistoryItemKeyFilterProperty.md)> | The unique key of the history item. | [optional]
**role** | Option<[**models::AgentInstanceHistoryRoleFilterProperty**](AgentInstanceHistoryRoleFilterProperty.md)> | The role of the history item. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The key of the element instance under which the history item was produced. | [optional]
**job_key** | Option<[**models::JobKeyFilterProperty**](JobKeyFilterProperty.md)> | The key of the job activation that produced the history item. | [optional]
**iteration** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The iteration number. | [optional]
**commit_status** | Option<[**models::AgentInstanceHistoryCommitStatusFilterProperty**](AgentInstanceHistoryCommitStatusFilterProperty.md)> | The commit status of the history item. Defaults to COMMITTED only. Include PENDING or DISCARDED explicitly to debug in-flight or failed activations.  | [optional]
**produced_at** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The timestamp when the history item was produced. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


