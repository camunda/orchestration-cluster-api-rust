# GroupFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The group ID search filters. | [optional]
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The group name search filters. | [optional]
**dollar_or** | Option<[**Vec<models::GroupFilterFields>**](GroupFilterFields.md)> | Defines a list of alternative filter groups combined using OR logic. Each object in the array is evaluated independently, and the filter matches if any one of them is satisfied.  Top-level fields and the `$or` clause are combined using AND logic — meaning: (top-level filters) AND (any of the `$or` filters) must match. <br> <em>Example:</em>  ```json {   \"$or\": [     { \"groupId\": \"group-1\" },     { \"groupId\": \"group-2\" }   ] } ``` This matches groups whose <code>groupId</code> is <em>group-1</em> or <em>group-2</em>. <br> <p>Note: Using complex <code>$or</code> conditions may impact performance, use with caution in high-volume environments.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


