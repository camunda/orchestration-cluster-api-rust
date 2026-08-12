# AgentInstanceCreatedHistoryItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**history_item_id** | **String** | The historyItemId of the corresponding item in the request, echoed back so callers can correlate response entries with request items by id.  | 
**history_item_key** | **models::AgentHistoryItemKey** | The system-generated key for the history item. When isDuplicate is true, this is the key of the original entry, not a new one.  | 
**is_duplicate** | **bool** | True if this item had already been recorded and no new AGENT_HISTORY event was created for it; false if a new event was created.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


