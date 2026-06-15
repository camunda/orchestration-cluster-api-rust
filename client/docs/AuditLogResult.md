# AuditLogResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**audit_log_key** | **models::AuditLogKey** | The unique key of the audit log entry. | 
**entity_key** | **String** | System-generated entity key for an audit log entry. | 
**entity_type** | [**models::AuditLogEntityTypeEnum**](AuditLogEntityTypeEnum.md) |  | 
**operation_type** | [**models::AuditLogOperationTypeEnum**](AuditLogOperationTypeEnum.md) |  | 
**batch_operation_key** | Option<**String**> | Key of the batch operation. | 
**batch_operation_type** | Option<[**models::BatchOperationTypeEnum**](BatchOperationTypeEnum.md)> | The type of batch operation performed, if this is part of a batch. | 
**timestamp** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the operation occurred. | 
**actor_id** | Option<**String**> | The ID of the actor who performed the operation. | 
**actor_type** | Option<[**models::AuditLogActorTypeEnum**](AuditLogActorTypeEnum.md)> | The type of the actor who performed the operation. | 
**agent_element_id** | Option<**String**> | The element ID of the agent that performed the operation (e.g. ad-hoc subprocess element ID). | 
**tenant_id** | Option<**String**> | The tenant ID of the audit log. | 
**result** | [**models::AuditLogResultEnum**](AuditLogResultEnum.md) |  | 
**category** | [**models::AuditLogCategoryEnum**](AuditLogCategoryEnum.md) |  | 
**process_definition_id** | Option<**String**> | The process definition ID. | 
**process_definition_key** | Option<**models::ProcessDefinitionKey**> | The key of the process definition. | 
**process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the process instance. | 
**root_process_instance_key** | Option<**models::ProcessInstanceKey**> | The key of the root process instance. The root process instance is the top-level ancestor in the process instance hierarchy. This field is only present for data belonging to process instance hierarchies created in version 8.9 or later.  | 
**element_instance_key** | Option<**models::ElementInstanceKey**> | The key of the element instance. | 
**job_key** | Option<**models::JobKey**> | The key of the job. | 
**user_task_key** | Option<**models::UserTaskKey**> | The key of the user task. | 
**decision_requirements_id** | Option<**String**> | The decision requirements ID. | 
**decision_requirements_key** | Option<**models::DecisionRequirementsKey**> | The assigned key of the decision requirements. | 
**decision_definition_id** | Option<**String**> | The decision definition ID. | 
**decision_definition_key** | Option<**models::DecisionDefinitionKey**> | The key of the decision definition. | 
**decision_evaluation_key** | Option<**models::DecisionEvaluationKey**> | The key of the decision evaluation. | 
**deployment_key** | Option<**models::DeploymentKey**> | The key of the deployment. | 
**form_key** | Option<**models::FormKey**> | The key of the form. | 
**resource_key** | Option<[**models::ResourceKey**](ResourceKey.md)> | The system-assigned key for this resource. | 
**related_entity_key** | Option<**String**> | The key of the related entity. The content depends on the operation type and entity type. For example, for authorization operations, this will contain the ID of the owner (e.g., user or group) the authorization belongs to.  | 
**related_entity_type** | Option<[**models::AuditLogEntityTypeEnum**](AuditLogEntityTypeEnum.md)> | The type of the related entity. The content depends on the operation type and entity type. For example, for authorization operations, this will contain the type of the owner (e.g., USER or GROUP) the authorization belongs to.  | 
**entity_description** | Option<**String**> | Additional description of the entity affected by the operation. For example, for variable operations, this will contain the variable name.  | 
**inbound_channel_type** | Option<**String**> | The type of the inbound channel that triggered the operation (e.g. MCP). | 
**inbound_channel_tool_name** | Option<**String**> | The tool name of the inbound channel (e.g. the MCP tool that triggered the operation). | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


