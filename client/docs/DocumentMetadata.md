# DocumentMetadata

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content_type** | Option<**String**> | The content type of the document. | [optional]
**file_name** | Option<**String**> | The name of the file. | [optional]
**expires_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The date and time when the document expires. | [optional]
**size** | Option<**i64**> | The size of the document in bytes. | [optional]
**process_definition_id** | Option<**String**> | The ID of the process definition that created the document. | [optional]
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the process instance that created the document. | [optional]
**custom_properties** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Custom properties of the document. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


