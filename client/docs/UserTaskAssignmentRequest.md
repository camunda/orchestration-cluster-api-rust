# UserTaskAssignmentRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**assignee** | Option<**String**> | The assignee for the user task. The assignee must not be empty or `null`. | [optional]
**allow_override** | Option<**bool**> | By default, the task is reassigned if it was already assigned. Set this to `false` to return an error in such cases. The task must then first be unassigned to be assigned again. Use this when you have users picking from group task queues to prevent race conditions.  | [optional]
**action** | Option<**String**> | A custom action value that will be accessible from user task events resulting from this endpoint invocation. If not provided, it will default to \"assign\".  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


