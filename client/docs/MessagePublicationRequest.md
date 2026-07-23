# MessagePublicationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of the message. | 
**correlation_key** | Option<**String**> | The correlation key of the message. | [optional][default to ]
**time_to_live** | Option<**i64**> | Timespan (in ms) to buffer the message on the broker. | [optional][default to 0]
**message_id** | Option<**String**> | The unique ID of the message. This is used to ensure only one message with the given ID will be published during the lifetime of the message (if `timeToLive` is set).  | [optional]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The message variables as JSON document. | [optional]
**tenant_id** | Option<**String**> | The tenant of the message sender. | [optional]
**business_id** | Option<**String**> | An optional business id used to enforce uniqueness of the process instance that a message start event would create. If provided and uniqueness enforcement is enabled, the engine rejects starting a new process instance when another root process instance with the same business id is already active for the same process definition. It has no effect when the message correlates to a catch, boundary, or intermediate event.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


