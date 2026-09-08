# RoleFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**role_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The role ID search filters. | [optional]
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The role name search filters. | [optional]
**dollar_or** | Option<[**Vec<models::RoleFilterFields>**](RoleFilterFields.md)> | Defines a list of alternative filter groups combined using OR logic. Each object in the array is evaluated independently, and the filter matches if any one of them is satisfied.  Top-level fields and the `$or` clause are combined using AND logic — meaning: (top-level filters) AND (any of the `$or` filters) must match. <br> <em>Example:</em>  ```json {   \"name\": \"Admin\",   \"$or\": [     { \"roleId\": \"role-1\" },     { \"roleId\": \"role-2\" }   ] } ``` This matches roles that:  <ul style=\"padding-left: 20px; margin-left: 20px;\">   <li style=\"list-style-type: disc;\">have name equal to <em>Admin</em></li>   <li style=\"list-style-type: disc;\">and match either:     <ul style=\"padding-left: 20px; margin-left: 20px;\">       <li style=\"list-style-type: circle;\"><code>roleId</code> is <em>role-1</em>, or</li>       <li style=\"list-style-type: circle;\"><code>roleId</code> is <em>role-2</em></li>     </ul>   </li> </ul> <br> <p>Note: Using complex <code>$or</code> conditions may impact performance, use with caution in high-volume environments.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


