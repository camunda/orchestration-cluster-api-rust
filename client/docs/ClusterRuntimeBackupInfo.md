# ClusterRuntimeBackupInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**backup_id** | **i64** | The id of the backup. | 
**state** | [**models::StateCode**](StateCode.md) | The state aggregated over every targeted physical tenant, whether the backup id was looked up directly or listed. A tenant holding nothing for this id counts as `DOES_NOT_EXIST`, so the aggregate is `INCOMPLETE` unless every targeted tenant holds the backup. | 
**failure_reason** | Option<**String**> | Reason for failure if the aggregated state is 'FAILED'. | 
**physical_tenants** | [**Vec<models::ClusterRuntimeBackupTenantInfo>**](ClusterRuntimeBackupTenantInfo.md) | What each physical tenant reports for this backup id, ordered by physical tenant id. Every targeted tenant is listed, including the ones reporting `DOES_NOT_EXIST`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


