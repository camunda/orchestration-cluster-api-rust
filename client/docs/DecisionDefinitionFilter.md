# DecisionDefinitionFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decision_definition_id** | Option<**String**> | The DMN ID of the decision definition. | [optional]
**name** | Option<**String**> | The DMN name of the decision definition. | [optional]
**is_latest_version** | Option<**bool**> | Whether to only return the latest version of each decision definition. When using this filter, pagination functionality is limited, you can only paginate forward using `after` and `limit`. The response contains no `startCursor` in the `page`, and requests ignore the `from` and `before` in the `page`.  | [optional]
**version** | Option<**i32**> | The assigned version of the decision definition. | [optional]
**decision_requirements_id** | Option<**String**> | the DMN ID of the decision requirements graph that the decision definition is part of. | [optional]
**tenant_id** | Option<**String**> | The tenant ID of the decision definition. | [optional]
**decision_definition_key** | Option<**models::DecisionDefinitionKey**> | The assigned key, which acts as a unique identifier for this decision definition. | [optional]
**decision_requirements_key** | Option<**models::DecisionRequirementsKey**> | The assigned key of the decision requirements graph that the decision definition is part of. | [optional]
**decision_requirements_name** | Option<**String**> | The DMN name of the decision requirements that the decision definition is part of. | [optional]
**decision_requirements_version** | Option<**i32**> | The assigned version of the decision requirements that the decision definition is part of. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


