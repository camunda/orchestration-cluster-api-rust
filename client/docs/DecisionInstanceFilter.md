# DecisionInstanceFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decision_evaluation_instance_key** | Option<[**models::DecisionEvaluationInstanceKeyFilterProperty**](DecisionEvaluationInstanceKeyFilterProperty.md)> | The key of the decision evaluation instance. | [optional]
**state** | Option<[**models::DecisionInstanceStateFilterProperty**](DecisionInstanceStateFilterProperty.md)> | The state of the decision instance. | [optional]
**evaluation_failure** | Option<**String**> | The evaluation failure of the decision instance. | [optional]
**evaluation_date** | Option<[**models::DateTimeFilterProperty**](DateTimeFilterProperty.md)> | The evaluation date of the decision instance. | [optional]
**decision_definition_id** | Option<**String**> | The ID of the DMN decision. | [optional]
**decision_definition_name** | Option<**String**> | The name of the DMN decision. | [optional]
**decision_definition_version** | Option<**i32**> | The version of the decision. | [optional]
**decision_definition_type** | Option<[**models::DecisionDefinitionTypeEnum**](DecisionDefinitionTypeEnum.md)> |  | [optional]
**tenant_id** | Option<**String**> | The tenant ID of the decision instance. | [optional]
**decision_evaluation_key** | Option<**models::DecisionEvaluationKey**> | The key of the parent decision evaluation. Note that this is not the identifier of an individual decision instance; the `decisionEvaluationInstanceKey` is the identifier for a decision instance.  | [optional]
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | The key of the process definition. | [optional]
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the process instance. | [optional]
**decision_definition_key** | Option<[**models::DecisionDefinitionKeyFilterProperty**](DecisionDefinitionKeyFilterProperty.md)> | The key of the decision. | [optional]
**element_instance_key** | Option<[**models::ElementInstanceKeyFilterProperty**](ElementInstanceKeyFilterProperty.md)> | The key of the element instance this decision instance is linked to. | [optional]
**root_decision_definition_key** | Option<[**models::DecisionDefinitionKeyFilterProperty**](DecisionDefinitionKeyFilterProperty.md)> | The key of the root decision definition. | [optional]
**decision_requirements_key** | Option<[**models::DecisionRequirementsKeyFilterProperty**](DecisionRequirementsKeyFilterProperty.md)> | The key of the decision requirements definition. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


