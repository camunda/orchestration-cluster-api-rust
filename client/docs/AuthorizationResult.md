# AuthorizationResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**owner_id** | **String** | The ID of the owner of permissions. | 
**owner_type** | [**models::OwnerTypeEnum**](OwnerTypeEnum.md) |  | 
**resource_type** | [**models::ResourceTypeEnum**](ResourceTypeEnum.md) | The type of resource that the permissions relate to. | 
**resource_id** | Option<**String**> | ID of the resource the permission relates to (mutually exclusive with `resourcePropertyName`). | 
**resource_property_name** | Option<**String**> | The name of the resource property the permission relates to (mutually exclusive with `resourceId`). | 
**permission_types** | [**Vec<models::PermissionTypeEnum>**](PermissionTypeEnum.md) | Specifies the types of the permissions. | 
**authorization_key** | **models::AuthorizationKey** | The key of the authorization. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


