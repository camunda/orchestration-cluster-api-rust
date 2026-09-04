# AgentInstanceHistoryItemMetricsRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**input_tokens** | Option<**i64**> | Input tokens consumed by this LLM call. Null when not provided. | [optional]
**output_tokens** | Option<**i64**> | Output tokens produced by this LLM call. Null when not provided. | [optional]
**reasoning_token_count** | Option<**i64**> | Reasoning tokens consumed by this LLM call. Null when not provided. | [optional]
**cache_creation_token_count** | Option<**i64**> | Cache-creation tokens consumed by this LLM call. Null when not provided. | [optional]
**cache_read_token_count** | Option<**i64**> | Cache-read tokens consumed by this LLM call. Null when not provided. | [optional]
**duration_ms** | Option<**i64**> | Wall-clock duration of the LLM call in milliseconds. Null when not provided. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


