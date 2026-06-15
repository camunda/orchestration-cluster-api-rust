# EvaluatedDecisionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decision_definition_id** | **String** | The ID of the decision which was evaluated. | 
**decision_definition_name** | **String** | The name of the decision which was evaluated. | 
**decision_definition_version** | **i32** | The version of the decision which was evaluated. | 
**decision_definition_type** | **String** | The type of the decision which was evaluated. | 
**output** | **String** | JSON document that will instantiate the result of the decision which was evaluated.  | 
**tenant_id** | **String** | The tenant ID of the evaluated decision. | 
**matched_rules** | [**Vec<models::MatchedDecisionRuleItem>**](MatchedDecisionRuleItem.md) | The decision rules that matched within this decision evaluation. | 
**evaluated_inputs** | [**Vec<models::EvaluatedDecisionInputItem>**](EvaluatedDecisionInputItem.md) | The decision inputs that were evaluated within this decision evaluation. | 
**decision_definition_key** | **models::DecisionDefinitionKey** | The unique key identifying the decision which was evaluate. | 
**decision_evaluation_instance_key** | **String** | The unique key identifying this decision evaluation instance. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


