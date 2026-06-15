# \DocumentApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_document**](DocumentApi.md#create_document) | **POST** /documents | Upload document
[**create_document_link**](DocumentApi.md#create_document_link) | **POST** /documents/{documentId}/links | Create document link
[**create_documents**](DocumentApi.md#create_documents) | **POST** /documents/batch | Upload multiple documents
[**delete_document**](DocumentApi.md#delete_document) | **DELETE** /documents/{documentId} | Delete document
[**get_document**](DocumentApi.md#get_document) | **GET** /documents/{documentId} | Download document



## create_document

> models::DocumentReference create_document(file, store_id, document_id, metadata)
Upload document

Upload a document to the Camunda 8 cluster.  Note that this is currently supported for document stores of type: AWS, Azure, GCP, in-memory (non-production), local (non-production) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**file** | **std::path::PathBuf** |  | [required] |
**store_id** | Option<**String**> | The ID of the document store to upload the documents to. Currently, only a single document store is supported per cluster. However, this attribute is included to allow for potential future support of multiple document stores. |  |
**document_id** | Option<**String**> | The ID of the document to upload. If not provided, a new ID will be generated. Specifying an existing ID will result in an error if the document already exists.  |  |
**metadata** | Option<[**models::DocumentMetadata**](DocumentMetadata.md)> |  |  |

### Return type

[**models::DocumentReference**](DocumentReference.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_document_link

> models::DocumentLink create_document_link(document_id, store_id, content_hash, document_link_request)
Create document link

Create a link to a document in the Camunda 8 cluster.  Note that this is currently supported for document stores of type: AWS, Azure, GCP 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**document_id** | **String** | The ID of the document to link. | [required] |
**store_id** | Option<**String**> | The ID of the document store where the document is located. |  |
**content_hash** | Option<**String**> | The hash of the document content that was computed by the document store during upload. The hash is part of the document reference that is returned when uploading a document. If the client fails to provide the correct hash, the request will be rejected.  |  |
**document_link_request** | Option<[**DocumentLinkRequest**](DocumentLinkRequest.md)> |  |  |

### Return type

[**models::DocumentLink**](DocumentLink.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_documents

> models::DocumentCreationBatchResponse create_documents(files, store_id, metadata_list)
Upload multiple documents

Upload multiple documents to the Camunda 8 cluster.  The caller must provide a file name for each document, which will be used in case of a multi-status response to identify which documents failed to upload. The file name can be provided in the `Content-Disposition` header of the file part or in the `fileName` field of the metadata. You can add a parallel array of metadata objects. These are matched with the files based on index, and must have the same length as the files array. To pass homogenous metadata for all files, spread the metadata over the metadata array. A filename value provided explicitly via the metadata array in the request overrides the `Content-Disposition` header of the file part.  In case of a multi-status response, the response body will contain a list of `DocumentBatchProblemDetail` objects, each of which contains the file name of the document that failed to upload and the reason for the failure. The client can choose to retry the whole batch or individual documents based on the response.  Note that this is currently supported for document stores of type: AWS, Azure, GCP, in-memory (non-production), local (non-production) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**files** | [**Vec<std::path::PathBuf>**](Std__path__PathBuf.md) | The documents to upload. | [required] |
**store_id** | Option<**String**> | The ID of the document store to upload the documents to. Currently, only a single document store is supported per cluster. However, this attribute is included to allow for potential future support of multiple document stores. |  |
**metadata_list** | Option<[**Vec<models::DocumentMetadata>**](Models__DocumentMetadata.md)> | Optional JSON array of metadata object whose index aligns with each file entry. The metadata array must have the same length as the files array.  |  |

### Return type

[**models::DocumentCreationBatchResponse**](DocumentCreationBatchResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_document

> delete_document(document_id, store_id)
Delete document

Delete a document from the Camunda 8 cluster.  Note that this is currently supported for document stores of type: AWS, Azure, GCP, in-memory (non-production), local (non-production) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**document_id** | **String** | The ID of the document to delete. | [required] |
**store_id** | Option<**String**> | The ID of the document store to delete the document from. |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_document

> std::path::PathBuf get_document(document_id, store_id, content_hash)
Download document

Download a document from the Camunda 8 cluster.  Note that this is currently supported for document stores of type: AWS, Azure, GCP, in-memory (non-production), local (non-production) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**document_id** | **String** | The ID of the document to download. | [required] |
**store_id** | Option<**String**> | The ID of the document store to download the document from. |  |
**content_hash** | Option<**String**> | The hash of the document content that was computed by the document store during upload. The hash is part of the document reference that is returned when uploading a document. If the client fails to provide the correct hash, the request will be rejected.  |  |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

