# CamundaUserResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**username** | **String** | The username of the user. | 
**display_name** | Option<**String**> | The display name of the user. | 
**email** | Option<**String**> | The email of the user. | 
**authorized_components** | **Vec<String>** | The web components the user is authorized to use. | 
**tenants** | [**Vec<models::TenantResult>**](TenantResult.md) | The tenants the user is a member of. | 
**groups** | **Vec<String>** | The groups assigned to the user. | 
**roles** | **Vec<String>** | The roles assigned to the user. | 
**sales_plan_type** | Option<**String**> | The plan of the user. | 
**c8_links** | **std::collections::HashMap<String, String>** | The links to the components in the C8 stack. | 
**can_logout** | **bool** | Flag for understanding if the user is able to perform logout. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


