# MessageCorrelationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The message name as defined in the BPMN process  | 
**correlation_key** | Option<**String**> | The correlation key of the message. | [optional][default to ]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The message variables as JSON document | [optional]
**tenant_id** | Option<**String**> | the tenant for which the message is published | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


