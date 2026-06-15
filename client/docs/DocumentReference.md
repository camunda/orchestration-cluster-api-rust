# DocumentReference

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**camunda_document_type** | **CamundaDocumentType** | Document discriminator. Always set to \"camunda\". (enum: camunda) | 
**store_id** | **String** | The ID of the document store. | 
**document_id** | **String** | The ID of the document. | 
**content_hash** | Option<**String**> | The hash of the document. | 
**metadata** | [**models::DocumentMetadataResponse**](DocumentMetadataResponse.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


