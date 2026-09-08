# MappingRuleFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**claim_name** | Option<**String**> | The claim name to match against a token. | [optional]
**claim_value** | Option<**String**> | The value of the claim to match. | [optional]
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The name of the mapping rule. | [optional]
**mapping_rule_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The ID of the mapping rule. | [optional]
**dollar_or** | Option<[**Vec<models::MappingRuleFilterFields>**](MappingRuleFilterFields.md)> | Defines a list of alternative filter groups combined using OR logic. Each object in the array is evaluated independently, and the filter matches if any one of them is satisfied.  Top-level fields and the `$or` clause are combined using AND logic — meaning: (top-level filters) AND (any of the `$or` filters) must match. <br> <em>Example:</em>  ```json {   \"$or\": [     { \"mappingRuleId\": \"rule-1\" },     { \"mappingRuleId\": \"rule-2\" }   ] } ``` This matches mapping rules whose <code>mappingRuleId</code> is <em>rule-1</em> or <em>rule-2</em>. <br> <p>Note: Using complex <code>$or</code> conditions may impact performance, use with caution in high-volume environments.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


