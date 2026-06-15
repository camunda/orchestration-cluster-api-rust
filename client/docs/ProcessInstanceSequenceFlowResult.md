# ProcessInstanceSequenceFlowResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**sequence_flow_id** | **String** | The sequence flow id. | 
**process_instance_key** | **models::ProcessInstanceKey** | The key of this process instance. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**process_definition_key** | **models::ProcessDefinitionKey** | The process definition key. | 
**process_definition_id** | **String** | The process definition id. | 
**element_id** | **String** | The element id for this sequence flow, as provided in the BPMN process. | 
**tenant_id** | **String** | The unique identifier of the tenant. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


