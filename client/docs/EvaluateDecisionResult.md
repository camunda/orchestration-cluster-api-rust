# EvaluateDecisionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decision_definition_id** | **String** | The ID of the decision which was evaluated. | 
**decision_definition_key** | **models::DecisionDefinitionKey** | The unique key identifying the decision which was evaluated. | 
**decision_definition_name** | **String** | The name of the decision which was evaluated. | 
**decision_definition_version** | **i32** | The version of the decision which was evaluated. | 
**decision_evaluation_key** | **models::DecisionEvaluationKey** | The unique key identifying this decision evaluation. | 
**decision_instance_key** | **models::DecisionInstanceKey** | Deprecated, please refer to `decisionEvaluationKey`. | 
**decision_requirements_id** | **String** | The ID of the decision requirements graph that the decision which was evaluated is part of. | 
**decision_requirements_key** | **models::DecisionRequirementsKey** | The unique key identifying the decision requirements graph that the decision which was evaluated is part of. | 
**evaluated_decisions** | [**Vec<models::EvaluatedDecisionResult>**](EvaluatedDecisionResult.md) | Decisions that were evaluated within the requested decision evaluation. | 
**failed_decision_definition_id** | Option<**String**> | The ID of the decision which failed during evaluation. | 
**failure_message** | Option<**String**> | Message describing why the decision which was evaluated failed. | 
**output** | **String** | JSON document that will instantiate the result of the decision which was evaluated.  | 
**tenant_id** | **String** | The tenant ID of the evaluated decision. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


