# ProcessDefinitionStatisticsFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**start_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The start date. | [optional]
**end_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The end date. | [optional]
**state** | Option<[**models::ProcessInstanceStateFilterProperty**](ProcessInstanceStateFilterProperty.md)> | The process instance state. | [optional]
**has_incident** | Option<**bool**> | Whether this process instance has a related incident or not. | [optional]
**suspended_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The time this process instance most recently entered the SUSPENDED state. This is cleared (null) again once the process instance is resumed.  | [optional]
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
**dollar_or** | Option<[**Vec<models::BaseProcessInstanceFilterFields>**](BaseProcessInstanceFilterFields.md)> | Defines a list of alternative filter groups combined using OR logic. Each object in the array is evaluated independently, and the filter matches if any one of them is satisfied.  Top-level fields and the `$or` clause are combined using AND logic — meaning: (top-level filters) AND (any of the `$or` filters) must match. <br> <em>Example:</em>  ```json {   \"state\": \"ACTIVE\",   \"tenantId\": 123,   \"$or\": [     { \"processDefinitionId\": \"process_v1\" },     { \"processDefinitionId\": \"process_v2\", \"hasIncident\": true }   ] } ``` This matches process instances that:  <ul style=\"padding-left: 20px; margin-left: 20px;\">   <li style=\"list-style-type: disc;\">are in <em>ACTIVE</em> state</li>   <li style=\"list-style-type: disc;\">have tenant id equal to <em>123</em></li>   <li style=\"list-style-type: disc;\">and match either:     <ul style=\"padding-left: 20px; margin-left: 20px;\">       <li style=\"list-style-type: circle;\"><code>processDefinitionId</code> is <em>process_v1</em>, or</li>       <li style=\"list-style-type: circle;\"><code>processDefinitionId</code> is <em>process_v2</em> and <code>hasIncident</code> is <em>true</em></li>     </ul>   </li> </ul> <br> <p>Note: Using complex <code>$or</code> conditions may impact performance, use with caution in high-volume environments.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


