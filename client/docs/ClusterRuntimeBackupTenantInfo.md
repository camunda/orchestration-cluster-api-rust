# ClusterRuntimeBackupTenantInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**physical_tenant_id** | **String** | The id of the physical tenant. | 
**state** | [**models::StateCode**](StateCode.md) | The state of the backup on this physical tenant, aggregated over its partitions. | 
**failure_reason** | Option<**String**> | Reason for failure if the state is 'FAILED'. | 
**details** | [**Vec<models::PartitionBackupInfo>**](PartitionBackupInfo.md) | Detailed status of the backup per partition of this physical tenant. Contains every partition of the tenant when the backup id was looked up directly, including for a tenant that holds no such backup. Empty for a tenant that holds nothing for a listed id: a listing asks each tenant for the backups it has, so there is nothing to report per partition for one it does not. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


