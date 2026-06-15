# UserTaskProperties

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **String** | The action performed on the user task. | 
**assignee** | Option<**String**> | The user assigned to the task. | 
**candidate_groups** | **Vec<String>** | The groups eligible to claim the task. | 
**candidate_users** | **Vec<String>** | The users eligible to claim the task. | 
**changed_attributes** | **Vec<String>** | The attributes that were changed in the task. | 
**due_date** | Option<**String**> | The due date of the user task in ISO 8601 format. | 
**follow_up_date** | Option<**String**> | The follow-up date of the user task in ISO 8601 format. | 
**form_key** | Option<**models::FormKey**> | The key of the form associated with the user task. | 
**priority** | Option<**i32**> | The priority of the user task. | 
**user_task_key** | Option<**models::UserTaskKey**> | The unique key identifying the user task. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


