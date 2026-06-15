# ProcessInstanceFilterFields

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**start_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The start date. | [optional]
**end_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The end date. | [optional]
**state** | Option<[**models::ProcessInstanceStateFilterProperty**](ProcessInstanceStateFilterProperty.md)> | The process instance state. | [optional]
**has_incident** | Option<**bool**> | Whether this process instance has a related incident or not. | [optional]
**tenant_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The tenant id. | [optional]
**variables** | Option<[**Vec<models::VariableValueFilterProperty>**](VariableValueFilterProperty.md)> | The process instance variables. | [optional]
**process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The key of this process instance. | [optional]
**parent_process_instance_key** | Option<[**models::ProcessInstanceKeyFilterProperty**](ProcessInstanceKeyFilterProperty.md)> | The parent process instance key. | [optional]
**parent_element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The parent element instance key. | [optional]
**batch_operation_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The batch operation id. **Deprecated**: Use `batchOperationKey` instead. This field will be removed in a future release. If both `batchOperationId` and `batchOperationKey` are provided, the request will be rejected with a 400 error.  | [optional]
**batch_operation_key** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The batch operation key. | [optional]
**error_message** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The error message related to the process. | [optional]
**has_retries_left** | Option<**bool**> | Whether the process has failed jobs with retries left. | [optional]
**element_instance_state** | Option<[**models::ElementInstanceStateFilterProperty**](ElementInstanceStateFilterProperty.md)> | The state of the element instances associated with the process instance. | [optional]
**element_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The element id associated with the process instance. | [optional]
**has_element_instance_incident** | Option<**bool**> | Whether the element instance has an incident or not. | [optional]
**incident_error_hash_code** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The incident error hash code, associated with this process. | [optional]
**tags** | Option<**HashSet<String>**> | List of tags. Tags need to start with a letter; then alphanumerics, `_`, `-`, `:`, or `.`; length ≤ 100. | [optional]
**business_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The business id associated with the process instance. | [optional]
**process_definition_id** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition id. | [optional]
**process_definition_name** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition name. | [optional]
**process_definition_version** | Option<[**models::IntegerFilterProperty**](IntegerFilterProperty.md)> | The process definition version. | [optional]
**process_definition_version_tag** | Option<[**models::StringFilterProperty**](StringFilterProperty.md)> | The process definition version tag. | [optional]
**process_definition_key** | Option<[**models::ProcessDefinitionKeyFilterProperty**](ProcessDefinitionKeyFilterProperty.md)> | The process definition key. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


