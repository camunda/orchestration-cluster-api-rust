# ProcessInstanceCreationInstructionByKey

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**process_definition_key** | **models::ProcessDefinitionKey** | The unique key identifying the process definition, for example, returned for a process in the deploy resources endpoint.  | 
**process_definition_version** | Option<**i32**> | As the version is already identified by the `processDefinitionKey`, the value of this field is ignored. It's here for backwards-compatibility only as previous releases accepted it in request bodies.  | [optional][default to -1]
**variables** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Set of variables as JSON object to instantiate in the root variable scope of the process instance. Can include nested complex objects.  | [optional]
**start_instructions** | Option<[**Vec<models::ProcessInstanceCreationStartInstruction>**](ProcessInstanceCreationStartInstruction.md)> | List of start instructions. By default, the process instance will start at the start event. If provided, the process instance will apply start instructions after it has been created.  | [optional]
**runtime_instructions** | Option<[**Vec<models::ProcessInstanceCreationTerminateInstruction>**](ProcessInstanceCreationTerminateInstruction.md)> | Runtime instructions (alpha). List of instructions that affect the runtime behavior of the process instance. Refer to specific instruction types for more details.  This parameter is an alpha feature and may be subject to change in future releases.  | [optional]
**tenant_id** | Option<**String**> | The tenant id of the process definition. If multi-tenancy is enabled, provide the tenant id of the process definition to start a process instance of. If multi-tenancy is disabled, don't provide this parameter.  | [optional]
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]
**await_completion** | Option<**bool**> | Wait for the process instance to complete. If the process instance does not complete within the request timeout limit, a 504 response status will be returned. The process instance will continue to run in the background regardless of the timeout. Disabled by default.  | [optional][default to false]
**request_timeout** | Option<**i64**> | Timeout (in ms) the request waits for the process to complete. By default or when set to 0, the generic request timeout configured in the cluster is applied.  | [optional][default to 0]
**fetch_variables** | Option<**Vec<String>**> | List of variables by name to be included in the response when awaitCompletion is set to true. If empty, all visible variables in the root scope will be returned.  | [optional]
**tags** | Option<**HashSet<String>**> | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | [optional]
**business_id** | Option<**String**> | An optional, user-defined string identifier that identifies the process instance within the scope of a process definition (scoped by tenant). If provided and uniqueness enforcement is enabled, the engine will reject creation if another root process instance with the same business id is already active for the same process definition. Note that any active child process instances with the same business id are not taken into account.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


