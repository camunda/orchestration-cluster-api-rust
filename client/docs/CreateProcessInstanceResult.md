# CreateProcessInstanceResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_id** | **String** | The BPMN process id of the process definition which was used to create the process. instance  | 
**process_definition_version** | **i32** | The version of the process definition which was used to create the process instance.  | 
**tenant_id** | **String** | The tenant id of the created process instance. | 
**variables** | **std::collections::HashMap<String, serde_json::Value>** | All the variables visible in the root scope. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition which was used to create the process instance.  | 
**process_instance_key** | **models::ProcessInstanceKey** | The unique identifier of the created process instance; to be used wherever a request needs a process instance key (e.g. CancelProcessInstanceRequest).  | 
**tags** | **HashSet<String>** | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | 
**business_id** | Option<**String**> | Business id as provided on creation. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


