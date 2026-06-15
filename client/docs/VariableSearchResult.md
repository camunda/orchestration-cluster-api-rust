# VariableSearchResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | Name of this variable. | 
**tenant_id** | **String** | Tenant ID of this variable. | 
**variable_key** | **models::VariableKey** | The key for this variable. | 
**scope_key** | [**models::ScopeKey**](ScopeKey.md) | The key of the scope where this variable is directly defined. For process-level variables, this is the process instance key. For local variables, this is the key of the specific element instance (task, subprocess, gateway, event, etc.) where the variable is directly defined.  | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of the process instance of this variable. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**value** | **String** | Value of this variable. Can be truncated. | 
**is_truncated** | **bool** | Whether the value is truncated or not. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


