# DeploymentDecisionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decision_definition_id** | **String** | The dmn decision ID, as parsed during deployment, together with the version forms a unique identifier for a specific decision.  | 
**version** | **i32** | The assigned decision version. | 
**name** | **String** | The DMN name of the decision, as parsed during deployment. | 
**tenant_id** | **String** | The tenant ID of the deployed decision. | 
**decision_requirements_id** | **String** | The dmn ID of the decision requirements graph that this decision is part of, as parsed during deployment.  | 
**decision_definition_key** | **models::DecisionDefinitionKey** | The assigned decision key, which acts as a unique identifier for this decision.  | 
**decision_requirements_key** | **models::DecisionRequirementsKey** | The assigned key of the decision requirements graph that this decision is part of.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


