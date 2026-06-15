# DocumentMetadataResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content_type** | **String** | The content type of the document. | 
**file_name** | **String** | The name of the file. | 
**expires_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The date and time when the document expires. | 
**size** | **i64** | The size of the document in bytes. | 
**process_definition_id** | Option<**String**> | The ID of the process definition that created the document. | 
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the process instance that created the document. | 
**custom_properties** | **std::collections::HashMap<String, serde_json::Value>** | Custom properties of the document. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


