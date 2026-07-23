# ProcessInstanceBusinessIdAssignmentInstruction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**business_id** | **String** | An optional, user-defined string identifier that identifies the process instance within the scope of a process definition (scoped by tenant). If provided and uniqueness enforcement is enabled, the engine will reject creation if another root process instance with the same business id is already active for the same process definition. Note that any active child process instances with the same business id are not taken into account.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


