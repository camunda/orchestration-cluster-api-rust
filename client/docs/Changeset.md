# Changeset

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**due_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The due date of the task. Reset by providing an empty String. | [optional]
**follow_up_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The follow-up date of the task. Reset by providing an empty String. | [optional]
**candidate_users** | Option<**Vec<String>**> | The list of candidate users of the task. Reset by providing an empty list. | [optional]
**candidate_groups** | Option<**Vec<String>**> | The list of candidate groups of the task. Reset by providing an empty list. | [optional]
**priority** | Option<**i32**> | The priority of the task. | [optional][default to 50]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


