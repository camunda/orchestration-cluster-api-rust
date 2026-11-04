# UserTaskLookupResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user_task_key** | **models::UserTaskKey** | The key of the user task. | 
**state** | [**models::UserTaskLookupStateEnum**](UserTaskLookupStateEnum.md) |  | 
**element_id** | **String** | The element ID of the user task. | 
**element_instance_key** | **models::ElementInstanceKey** | The key of the element instance. | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of the process instance. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**process_definition_id** | **String** | The ID of the process definition. | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition. | 
**process_definition_version** | **i32** | The version of the process definition. | 
**tenant_id** | **String** | The unique identifier of the tenant. | 
**assignee** | Option<**String**> | The assignee of the user task. While the task is in the `CREATING` state, the assignee from the process model is not set yet.  | 
**candidate_groups** | **Vec<String>** | The candidate groups for this user task. | 
**candidate_users** | **Vec<String>** | The candidate users for this user task. | 
**due_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The due date of the user task. | 
**follow_up_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The follow-up date of the user task. | 
**form_key** | Option<**models::FormKey**> | The key of the form. | 
**external_form_reference** | Option<**String**> | The external form reference. | 
**custom_headers** | **std::collections::HashMap<String, String>** | Custom headers for the user task. | 
**priority** | **i32** | The priority of the user task. The higher the value the higher the priority. | 
**tags** | **HashSet<String>** | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | 
**business_id** | Option<**String**> | The business ID of the owning process instance, inherited when the user task was created. This is `null` for user tasks whose owning process instance has no business ID.  | 
**creation_date** | **chrono::DateTime<chrono::FixedOffset>** | The creation date of the user task. | 
**variables** | **std::collections::HashMap<String, serde_json::Value>** | The variables visible to the user task, as a JSON object of name and value. If a name exists in several scopes, the value of the innermost scope is returned. Values are never truncated.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


