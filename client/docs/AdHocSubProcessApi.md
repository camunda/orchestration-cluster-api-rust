# \AdHocSubProcessApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**activate_ad_hoc_sub_process_activities**](AdHocSubProcessApi.md#activate_ad_hoc_sub_process_activities) | **POST** /element-instances/ad-hoc-activities/{adHocSubProcessInstanceKey}/activation | Activate activities within an ad-hoc sub-process



## activate_ad_hoc_sub_process_activities

> activate_ad_hoc_sub_process_activities(ad_hoc_sub_process_instance_key, ad_hoc_sub_process_activate_activities_instruction)
Activate activities within an ad-hoc sub-process

Activates selected activities within an ad-hoc sub-process identified by element ID. The provided element IDs must exist within the ad-hoc sub-process instance identified by the provided adHocSubProcessInstanceKey. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ad_hoc_sub_process_instance_key** | **ElementInstanceKey** | The key of the ad-hoc sub-process instance that contains the activities. | [required] |
**ad_hoc_sub_process_activate_activities_instruction** | [**AdHocSubProcessActivateActivitiesInstruction**](AdHocSubProcessActivateActivitiesInstruction.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

