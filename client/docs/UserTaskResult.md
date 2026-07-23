# UserTaskResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<**String**> | The name for this user task. | 
**state** | [**models::UserTaskStateEnum**](UserTaskStateEnum.md) |  | 
**assignee** | Option<**String**> | The assignee of the user task. | 
**element_id** | **String** | The element ID of the user task. | 
**candidate_groups** | **Vec<String>** | The candidate groups for this user task. | 
**candidate_users** | **Vec<String>** | The candidate users for this user task. | 
**process_definition_id** | **String** | The ID of the process definition. | 
**creation_date** | **chrono::DateTime<chrono::FixedOffset>** | The creation date of a user task. | 
**completion_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The completion date of a user task. | 
**follow_up_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The follow date of a user task. | 
**due_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The due date of a user task. | 
**tenant_id** | **String** | The unique identifier of the tenant. | 
**external_form_reference** | Option<**String**> | The external form reference. | 
**process_definition_version** | **i32** | The version of the process definition. | 
**custom_headers** | **std::collections::HashMap<String, String>** | Custom headers for the user task. | 
**priority** | **i32** | The priority of a user task. The higher the value the higher the priority. | [default to 50]
**user_task_key** | **models::UserTaskKey** | The key of the user task. | 
**element_instance_key** | **models::ElementInstanceKey** | The key of the element instance. | 
**process_name** | Option<**String**> | The name of the process definition. This is `null` if the process has no name defined.  | 
**process_definition_key** | **models::ProcessDefinitionKey** | The key of the process definition. | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of the process instance. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**business_id** | Option<**String**> | The business ID of the owning process instance, inherited when the user task was created. This is `null` for user tasks created before version 8.10, and for user tasks whose owning process instance has no business ID.  | 
**form_key** | Option<**models::FormKey**> | The key of the form. | 
**tags** | **HashSet<String>** | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


