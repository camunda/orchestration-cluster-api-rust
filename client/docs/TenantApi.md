# \TenantApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assign_client_to_tenant**](TenantApi.md#assign_client_to_tenant) | **PUT** /tenants/{tenantId}/clients/{clientId} | Assign a client to a tenant
[**assign_group_to_tenant**](TenantApi.md#assign_group_to_tenant) | **PUT** /tenants/{tenantId}/groups/{groupId} | Assign a group to a tenant
[**assign_mapping_rule_to_tenant**](TenantApi.md#assign_mapping_rule_to_tenant) | **PUT** /tenants/{tenantId}/mapping-rules/{mappingRuleId} | Assign a mapping rule to a tenant
[**assign_role_to_tenant**](TenantApi.md#assign_role_to_tenant) | **PUT** /tenants/{tenantId}/roles/{roleId} | Assign a role to a tenant
[**assign_user_to_tenant**](TenantApi.md#assign_user_to_tenant) | **PUT** /tenants/{tenantId}/users/{username} | Assign a user to a tenant
[**create_tenant**](TenantApi.md#create_tenant) | **POST** /tenants | Create tenant
[**delete_tenant**](TenantApi.md#delete_tenant) | **DELETE** /tenants/{tenantId} | Delete tenant
[**get_tenant**](TenantApi.md#get_tenant) | **GET** /tenants/{tenantId} | Get tenant
[**search_clients_for_tenant**](TenantApi.md#search_clients_for_tenant) | **POST** /tenants/{tenantId}/clients/search | Search clients for tenant
[**search_group_ids_for_tenant**](TenantApi.md#search_group_ids_for_tenant) | **POST** /tenants/{tenantId}/groups/search | Search groups for tenant
[**search_mapping_rules_for_tenant**](TenantApi.md#search_mapping_rules_for_tenant) | **POST** /tenants/{tenantId}/mapping-rules/search | Search mapping rules for tenant
[**search_roles_for_tenant**](TenantApi.md#search_roles_for_tenant) | **POST** /tenants/{tenantId}/roles/search | Search roles for tenant
[**search_tenants**](TenantApi.md#search_tenants) | **POST** /tenants/search | Search tenants
[**search_users_for_tenant**](TenantApi.md#search_users_for_tenant) | **POST** /tenants/{tenantId}/users/search | Search users for tenant
[**unassign_client_from_tenant**](TenantApi.md#unassign_client_from_tenant) | **DELETE** /tenants/{tenantId}/clients/{clientId} | Unassign a client from a tenant
[**unassign_group_from_tenant**](TenantApi.md#unassign_group_from_tenant) | **DELETE** /tenants/{tenantId}/groups/{groupId} | Unassign a group from a tenant
[**unassign_mapping_rule_from_tenant**](TenantApi.md#unassign_mapping_rule_from_tenant) | **DELETE** /tenants/{tenantId}/mapping-rules/{mappingRuleId} | Unassign a mapping rule from a tenant
[**unassign_role_from_tenant**](TenantApi.md#unassign_role_from_tenant) | **DELETE** /tenants/{tenantId}/roles/{roleId} | Unassign a role from a tenant
[**unassign_user_from_tenant**](TenantApi.md#unassign_user_from_tenant) | **DELETE** /tenants/{tenantId}/users/{username} | Unassign a user from a tenant
[**update_tenant**](TenantApi.md#update_tenant) | **PUT** /tenants/{tenantId} | Update tenant



## assign_client_to_tenant

> assign_client_to_tenant(tenant_id, client_id)
Assign a client to a tenant

Assign the client to the specified tenant. The client can then access tenant data and perform authorized actions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**client_id** | **String** | The unique identifier of the application. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_group_to_tenant

> assign_group_to_tenant(tenant_id, group_id)
Assign a group to a tenant

Assigns a group to a specified tenant. Group members (users, clients) can then access tenant data and perform authorized actions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**group_id** | **String** | The unique identifier of the group. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_mapping_rule_to_tenant

> assign_mapping_rule_to_tenant(tenant_id, mapping_rule_id)
Assign a mapping rule to a tenant

Assign a single mapping rule to a specified tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**mapping_rule_id** | **String** | The unique identifier of the mapping rule. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_role_to_tenant

> assign_role_to_tenant(tenant_id, role_id)
Assign a role to a tenant

Assigns a role to a specified tenant. Users, Clients or Groups, that have the role assigned, will get access to the tenant's data and can perform actions according to their authorizations. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**role_id** | **String** | The unique identifier of the role. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_user_to_tenant

> assign_user_to_tenant(tenant_id, username)
Assign a user to a tenant

Assign a single user to a specified tenant. The user can then access tenant data and perform authorized actions.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**username** | **String** | The unique identifier of the user. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_tenant

> models::TenantCreateResult create_tenant(tenant_create_request)
Create tenant

Creates a new tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_create_request** | [**TenantCreateRequest**](TenantCreateRequest.md) |  | [required] |

### Return type

[**models::TenantCreateResult**](TenantCreateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_tenant

> delete_tenant(tenant_id)
Delete tenant

Deletes an existing tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant

> models::TenantResult get_tenant(tenant_id)
Get tenant

Retrieves a single tenant by tenant ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |

### Return type

[**models::TenantResult**](TenantResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_clients_for_tenant

> models::TenantClientSearchResult search_clients_for_tenant(tenant_id, tenant_client_search_query_request)
Search clients for tenant

Retrieves a filtered and sorted list of clients for a specified tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**tenant_client_search_query_request** | Option<[**TenantClientSearchQueryRequest**](TenantClientSearchQueryRequest.md)> |  |  |

### Return type

[**models::TenantClientSearchResult**](TenantClientSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_group_ids_for_tenant

> models::TenantGroupSearchResult search_group_ids_for_tenant(tenant_id, tenant_group_search_query_request)
Search groups for tenant

Retrieves a filtered and sorted list of groups for a specified tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**tenant_group_search_query_request** | Option<[**TenantGroupSearchQueryRequest**](TenantGroupSearchQueryRequest.md)> |  |  |

### Return type

[**models::TenantGroupSearchResult**](TenantGroupSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_mapping_rules_for_tenant

> models::TenantMappingRuleSearchResult search_mapping_rules_for_tenant(tenant_id, mapping_rule_search_query_request)
Search mapping rules for tenant

Retrieves a filtered and sorted list of MappingRules for a specified tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**mapping_rule_search_query_request** | Option<[**MappingRuleSearchQueryRequest**](MappingRuleSearchQueryRequest.md)> |  |  |

### Return type

[**models::TenantMappingRuleSearchResult**](TenantMappingRuleSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_roles_for_tenant

> models::TenantRoleSearchResult search_roles_for_tenant(tenant_id, role_search_query_request)
Search roles for tenant

Retrieves a filtered and sorted list of roles for a specified tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**role_search_query_request** | Option<[**RoleSearchQueryRequest**](RoleSearchQueryRequest.md)> |  |  |

### Return type

[**models::TenantRoleSearchResult**](TenantRoleSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_tenants

> models::TenantSearchQueryResult search_tenants(tenant_search_query_request)
Search tenants

Retrieves a filtered and sorted list of tenants.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_search_query_request** | Option<[**TenantSearchQueryRequest**](TenantSearchQueryRequest.md)> |  |  |

### Return type

[**models::TenantSearchQueryResult**](TenantSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_users_for_tenant

> models::TenantUserSearchResult search_users_for_tenant(tenant_id, tenant_user_search_query_request)
Search users for tenant

Retrieves a filtered and sorted list of users for a specified tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**tenant_user_search_query_request** | Option<[**TenantUserSearchQueryRequest**](TenantUserSearchQueryRequest.md)> |  |  |

### Return type

[**models::TenantUserSearchResult**](TenantUserSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_client_from_tenant

> unassign_client_from_tenant(tenant_id, client_id)
Unassign a client from a tenant

Unassigns the client from the specified tenant. The client can no longer access tenant data. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**client_id** | **String** | The unique identifier of the application. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_group_from_tenant

> unassign_group_from_tenant(tenant_id, group_id)
Unassign a group from a tenant

Unassigns a group from a specified tenant. Members of the group (users, clients) will no longer have access to the tenant's data - except they are assigned directly to the tenant. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**group_id** | **String** | The unique identifier of the group. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_mapping_rule_from_tenant

> unassign_mapping_rule_from_tenant(tenant_id, mapping_rule_id)
Unassign a mapping rule from a tenant

Unassigns a single mapping rule from a specified tenant without deleting the rule.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**mapping_rule_id** | **String** | The unique identifier of the mapping rule. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_role_from_tenant

> unassign_role_from_tenant(tenant_id, role_id)
Unassign a role from a tenant

Unassigns a role from a specified tenant. Users, Clients or Groups, that have the role assigned, will no longer have access to the tenant's data - unless they are assigned directly to the tenant. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**role_id** | **String** | The unique identifier of the role. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_user_from_tenant

> unassign_user_from_tenant(tenant_id, username)
Unassign a user from a tenant

Unassigns the user from the specified tenant. The user can no longer access tenant data. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**username** | **String** | The unique identifier of the user. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant

> models::TenantUpdateResult update_tenant(tenant_id, tenant_update_request)
Update tenant

Updates an existing tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | The unique identifier of the tenant. | [required] |
**tenant_update_request** | [**TenantUpdateRequest**](TenantUpdateRequest.md) |  | [required] |

### Return type

[**models::TenantUpdateResult**](TenantUpdateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

