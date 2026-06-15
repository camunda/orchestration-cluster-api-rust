# \DecisionDefinitionApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**evaluate_decision**](DecisionDefinitionApi.md#evaluate_decision) | **POST** /decision-definitions/evaluation | Evaluate decision
[**get_decision_definition**](DecisionDefinitionApi.md#get_decision_definition) | **GET** /decision-definitions/{decisionDefinitionKey} | Get decision definition
[**get_decision_definition_xml**](DecisionDefinitionApi.md#get_decision_definition_xml) | **GET** /decision-definitions/{decisionDefinitionKey}/xml | Get decision definition XML
[**search_decision_definitions**](DecisionDefinitionApi.md#search_decision_definitions) | **POST** /decision-definitions/search | Search decision definitions



## evaluate_decision

> models::EvaluateDecisionResult evaluate_decision(decision_evaluation_instruction)
Evaluate decision

Evaluates a decision. You specify the decision to evaluate either by using its unique key (as returned by DeployResource), or using the decision ID. When using the decision ID, the latest deployed version of the decision is used. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_evaluation_instruction** | [**DecisionEvaluationInstruction**](DecisionEvaluationInstruction.md) |  | [required] |

### Return type

[**models::EvaluateDecisionResult**](EvaluateDecisionResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_decision_definition

> models::DecisionDefinitionResult get_decision_definition(decision_definition_key)
Get decision definition

Returns a decision definition by key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_definition_key** | **DecisionDefinitionKey** | The assigned key of the decision definition, which acts as a unique identifier for this decision. | [required] |

### Return type

[**models::DecisionDefinitionResult**](DecisionDefinitionResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_decision_definition_xml

> String get_decision_definition_xml(decision_definition_key)
Get decision definition XML

Returns decision definition as XML.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_definition_key** | **DecisionDefinitionKey** | The assigned key of the decision definition, which acts as a unique identifier for this decision. | [required] |

### Return type

**String**

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/xml, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_decision_definitions

> models::DecisionDefinitionSearchQueryResult search_decision_definitions(decision_definition_search_query)
Search decision definitions

Search for decision definitions based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decision_definition_search_query** | Option<[**DecisionDefinitionSearchQuery**](DecisionDefinitionSearchQuery.md)> |  |  |

### Return type

[**models::DecisionDefinitionSearchQueryResult**](DecisionDefinitionSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

