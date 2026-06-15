# ProcessDefinitionFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Name of this process definition. | [optional]
**is_latest_version** | Option<**bool**> | Whether to only return the latest version of each process definition. When using this filter, pagination functionality is limited, you can only paginate forward using `after` and `limit`. The response contains no `startCursor` in the `page`, and requests ignore the `from` and `before` in the `page`. When using this filter, sorting is limited to `processDefinitionId` and `tenantId` fields only.  | [optional]
**resource_name** | Option<**String**> | Resource name of this process definition. | [optional]
**version** | Option<**i32**> | Version of this process definition. | [optional]
**version_tag** | Option<**String**> | Version tag of this process definition. | [optional]
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Process definition ID of this process definition. | [optional]
**tenant_id** | Option<**String**> | Tenant ID of this process definition. | [optional]
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | The key for this process definition. | [optional]
**has_start_form** | Option<**bool**> | Indicates whether the start event of the process has an associated Form Key. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


