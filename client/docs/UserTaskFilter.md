# UserTaskFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | Option<[**models::UserTaskStateFilterProperty**](UserTaskStateFilterProperty.md)> | The user task state. | [optional]
**is_suspended** | Option<**bool**> | Whether the user task is suspended because its owning process instance is suspended.  | [optional]
**assignee** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The assignee of the user task. | [optional]
**business_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The business ID of the owning process instance the user task belongs to. This only works for user tasks created with 8.10 and onwards. Tasks from prior versions don't contain this data and cannot be found.  | [optional]
**priority** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The priority of the user task. | [optional]
**element_id** | Option<**String**> | The element ID of the user task. | [optional]
**name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The task name. This only works for data created with 8.8 and onwards. Instances from prior versions don't contain this data and cannot be found.  | [optional]
**candidate_group** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The candidate group for this user task. | [optional]
**candidate_user** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The candidate user for this user task. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | Tenant ID of this user task. | [optional]
**process_definition_id** | Option<[**models::ProcessDefinitionIdFilterProperty**](ProcessDefinitionIdFilterProperty.md)> | The ID of the process definition. | [optional]
**creation_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The user task creation date. | [optional]
**completion_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The user task completion date. | [optional]
**follow_up_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The user task follow-up date. | [optional]
**due_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The user task due date. | [optional]
**process_instance_variables** | Option<[**Vec<models::VariableValueFilterProperty>**](VariableValueFilterProperty.md)> | The variables of the process instance. | [optional]
**local_variables** | Option<[**Vec<models::VariableValueFilterProperty>**](VariableValueFilterProperty.md)> | The local variables of the user task. | [optional]
**user_task_key** | Option<**models::UserTaskKey**> | The key for this user task. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The key of the process definition. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The key of the process instance. | [optional]
**element_instance_key** | Option<**models::ElementInstanceKey**> | The key of the element instance. | [optional]
**tags** | Option<**HashSet<String>**> | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | [optional]
**dollar_or** | Option<[**Vec<models::UserTaskFilterFields>**](UserTaskFilterFields.md)> | Defines a list of alternative filter groups combined using OR logic. Each object in the array is evaluated independently, and the filter matches if any one of them is satisfied.  Top-level fields and the `$or` clause are combined using AND logic — meaning: (top-level filters) AND (any of the `$or` filters) must match. <br> <em>Example:</em>  ```json {   \"assignee\": \"user1\",   \"$or\": [     { \"candidateGroup\": \"groupA\" },     { \"candidateUser\": \"user2\" }   ] } ``` This matches user tasks that:  <ul style=\"padding-left: 20px; margin-left: 20px;\">   <li style=\"list-style-type: disc;\">are assigned to <em>user1</em></li>   <li style=\"list-style-type: disc;\">and match either:     <ul style=\"padding-left: 20px; margin-left: 20px;\">       <li style=\"list-style-type: circle;\"><code>candidateGroup</code> is <em>groupA</em>, or</li>       <li style=\"list-style-type: circle;\"><code>candidateUser</code> is <em>user2</em></li>     </ul>   </li> </ul> <br> <p>Note: Using complex <code>$or</code> conditions may impact performance, use with caution in high-volume environments.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


