# DecisionInstanceResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decision_definition_id** | **String** | The ID of the DMN decision. | 
**decision_definition_key** | **models::DecisionDefinitionKey** | The key of the decision. | 
**decision_definition_name** | **String** | The name of the DMN decision. | 
**decision_definition_type** | [**models::DecisionDefinitionTypeEnum**](DecisionDefinitionTypeEnum.md) |  | 
**decision_definition_version** | **i32** | The version of the decision. | 
**decision_evaluation_instance_key** | **String** | System-generated identifier for a decision evaluation instance. It is composed of the parent decision evaluation key and the 1-based index of the evaluated decision within that evaluation, joined by a hyphen (format: `<decisionEvaluationKey>-<index>`).  | 
**decision_evaluation_key** | **models::DecisionEvaluationKey** | The key of the decision evaluation where this instance was created. | 
**element_instance_key** | Option<**models::ElementInstanceKey**> | The key of the element instance this decision instance is linked to. | 
**evaluation_date** | **chrono::DateTime<chrono::FixedOffset>** | The evaluation date of the decision instance. | 
**evaluation_failure** | Option<**String**> | The evaluation failure of the decision instance. | 
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | The key of the process definition. | 
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the process instance. | 
**result** | **String** | The result of the decision instance. | 
**root_decision_definition_key** | **models::DecisionDefinitionKey** | The key of the root decision definition. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**state** | [**models::DecisionInstanceStateEnum**](DecisionInstanceStateEnum.md) |  | 
**tenant_id** | **String** | The tenant ID of the decision instance. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


