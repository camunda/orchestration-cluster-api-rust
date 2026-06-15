# VariableFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Name of the variable. | [optional]
**value** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The value of the variable. Variable values in filters need to be in serialized JSON format. For example, a variable with string value `myValue` can be found with the filter value `\"myValue\"`. Consider appropriate escaping for special characters in JSON strings when constructing filter values.  | [optional]
**tenant_id** | Option<**String**> | Tenant ID of this variable. | [optional]
**is_truncated** | Option<**bool**> | Whether the value is truncated or not. | [optional]
**variable_key** | Option<[**models::VariableKeyFilterProperty**](VariableKeyFilterProperty.md)> | The key for this variable. | [optional]
**scope_key** | Option<[**models::ScopeKeyFilterProperty**](ScopeKeyFilterProperty.md)> | The key of the scope that defines where this variable is directly defined. This can be a process instance key (for process-level variables) or an element instance key (for local variables scoped to tasks, subprocesses, gateways, events, etc.). Use this filter to find variables directly defined in specific scopes. Note that this does not include variables from parent scopes that would be visible through the scope hierarchy.  | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The key of the process instance of this variable. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


