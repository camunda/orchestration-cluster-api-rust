# UserFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**username** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The username of the user. | [optional]
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The name of the user. | [optional]
**email** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The email of the user. | [optional]
**dollar_or** | Option<[**Vec<models::UserFilterFields>**](UserFilterFields.md)> | Defines a list of alternative filter groups combined using OR logic. Each object in the array is evaluated independently, and the filter matches if any one of them is satisfied.  Top-level fields and the `$or` clause are combined using AND logic — meaning: (top-level filters) AND (any of the `$or` filters) must match. <br> <em>Example:</em>  ```json {   \"$or\": [     { \"username\": \"user-1\" },     { \"username\": \"user-2\" }   ] } ``` This matches users whose <code>username</code> is <em>user-1</em> or <em>user-2</em>. <br> <p>Note: Using complex <code>$or</code> conditions may impact performance, use with caution in high-volume environments.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


