# MessageCorrelationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The message name as defined in the BPMN process  | 
**correlation_key** | Option<**String**> | The correlation key of the message. | [optional][default to ]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The message variables as JSON document | [optional]
**tenant_id** | Option<**String**> | the tenant for which the message is published | [optional]
**business_id** | Option<**String**> | An optional business id used to enforce uniqueness of the process instance that a message start event would create. If provided and uniqueness enforcement is enabled, the engine rejects starting a new process instance when another root process instance with the same business id is already active for the same process definition. It has no effect when the message correlates to a catch, boundary, or intermediate event.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


