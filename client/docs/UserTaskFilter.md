# UserTaskFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | Option<[**models::UserTaskStateFilterProperty**](UserTaskStateFilterProperty.md)> | The user task state. | [optional]
**assignee** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The assignee of the user task. | [optional]
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

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


