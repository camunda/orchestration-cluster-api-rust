# ElementInstanceWaitStateResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | Key of the root process instance. | 
**process_instance_key** | **models::ProcessInstanceKey** | The process instance key associated to this element instance. | 
**element_instance_key** | **models::ElementInstanceKey** | The element instance key associated to this element instance. | 
**element_id** | **String** | The element ID for this element instance. | 
**element_type** | [**models::WaitStateElementTypeEnum**](WaitStateElementTypeEnum.md) | The BPMN element type of this element instance. | 
**tenant_id** | **String** | The tenant ID of the element instance. | 
**bpmn_process_id** | **String** | The BPMN process ID of the process definition associated to this element instance. | 
**details** | [**models::WaitStateDetails**](WaitStateDetails.md) | Wait-state-specific details, resolved by waitStateType. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


