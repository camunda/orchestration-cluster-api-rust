# \RoleApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assign_role_to_client**](RoleApi.md#assign_role_to_client) | **PUT** /roles/{roleId}/clients/{clientId} | Assign a role to a client
[**assign_role_to_group**](RoleApi.md#assign_role_to_group) | **PUT** /roles/{roleId}/groups/{groupId} | Assign a role to a group
[**assign_role_to_mapping_rule**](RoleApi.md#assign_role_to_mapping_rule) | **PUT** /roles/{roleId}/mapping-rules/{mappingRuleId} | Assign a role to a mapping rule
[**assign_role_to_user**](RoleApi.md#assign_role_to_user) | **PUT** /roles/{roleId}/users/{username} | Assign a role to a user
[**create_role**](RoleApi.md#create_role) | **POST** /roles | Create role
[**delete_role**](RoleApi.md#delete_role) | **DELETE** /roles/{roleId} | Delete role
[**get_role**](RoleApi.md#get_role) | **GET** /roles/{roleId} | Get role
[**search_clients_for_role**](RoleApi.md#search_clients_for_role) | **POST** /roles/{roleId}/clients/search | Search role clients
[**search_groups_for_role**](RoleApi.md#search_groups_for_role) | **POST** /roles/{roleId}/groups/search | Search role groups
[**search_mapping_rules_for_role**](RoleApi.md#search_mapping_rules_for_role) | **POST** /roles/{roleId}/mapping-rules/search | Search role mapping rules
[**search_roles**](RoleApi.md#search_roles) | **POST** /roles/search | Search roles
[**search_users_for_role**](RoleApi.md#search_users_for_role) | **POST** /roles/{roleId}/users/search | Search role users
[**unassign_role_from_client**](RoleApi.md#unassign_role_from_client) | **DELETE** /roles/{roleId}/clients/{clientId} | Unassign a role from a client
[**unassign_role_from_group**](RoleApi.md#unassign_role_from_group) | **DELETE** /roles/{roleId}/groups/{groupId} | Unassign a role from a group
[**unassign_role_from_mapping_rule**](RoleApi.md#unassign_role_from_mapping_rule) | **DELETE** /roles/{roleId}/mapping-rules/{mappingRuleId} | Unassign a role from a mapping rule
[**unassign_role_from_user**](RoleApi.md#unassign_role_from_user) | **DELETE** /roles/{roleId}/users/{username} | Unassign a role from a user
[**update_role**](RoleApi.md#update_role) | **PUT** /roles/{roleId} | Update role



## assign_role_to_client

> assign_role_to_client(role_id, client_id)
Assign a role to a client

Assigns the specified role to the client. The client will inherit the authorizations associated with this role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**client_id** | **String** | The client ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_role_to_group

> assign_role_to_group(role_id, group_id)
Assign a role to a group

Assigns the specified role to the group. Every member of the group (user or client) will inherit the authorizations associated with this role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**group_id** | **String** | The group ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_role_to_mapping_rule

> assign_role_to_mapping_rule(role_id, mapping_rule_id)
Assign a role to a mapping rule

Assigns a role to a mapping rule.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**mapping_rule_id** | **String** | The mapping rule ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_role_to_user

> assign_role_to_user(role_id, username)
Assign a role to a user

Assigns the specified role to the user. The user will inherit the authorizations associated with this role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**username** | **String** | The user username. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_role

> models::RoleCreateResult create_role(role_create_request)
Create role

Create a new role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_create_request** | Option<[**RoleCreateRequest**](RoleCreateRequest.md)> |  |  |

### Return type

[**models::RoleCreateResult**](RoleCreateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_role

> delete_role(role_id)
Delete role

Deletes the role with the given ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_role

> models::RoleResult get_role(role_id)
Get role

Get a role by its ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |

### Return type

[**models::RoleResult**](RoleResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_clients_for_role

> models::RoleClientSearchResult search_clients_for_role(role_id, role_client_search_query_request)
Search role clients

Search clients with assigned role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**role_client_search_query_request** | Option<[**RoleClientSearchQueryRequest**](RoleClientSearchQueryRequest.md)> |  |  |

### Return type

[**models::RoleClientSearchResult**](RoleClientSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_groups_for_role

> models::RoleGroupSearchResult search_groups_for_role(role_id, role_group_search_query_request)
Search role groups

Search groups with assigned role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**role_group_search_query_request** | Option<[**RoleGroupSearchQueryRequest**](RoleGroupSearchQueryRequest.md)> |  |  |

### Return type

[**models::RoleGroupSearchResult**](RoleGroupSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_mapping_rules_for_role

> models::RoleMappingRuleSearchResult search_mapping_rules_for_role(role_id, mapping_rule_search_query_request)
Search role mapping rules

Search mapping rules with assigned role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**mapping_rule_search_query_request** | Option<[**MappingRuleSearchQueryRequest**](MappingRuleSearchQueryRequest.md)> |  |  |

### Return type

[**models::RoleMappingRuleSearchResult**](RoleMappingRuleSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_roles

> models::RoleSearchQueryResult search_roles(role_search_query_request)
Search roles

Search for roles based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_search_query_request** | Option<[**RoleSearchQueryRequest**](RoleSearchQueryRequest.md)> |  |  |

### Return type

[**models::RoleSearchQueryResult**](RoleSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_users_for_role

> models::RoleUserSearchResult search_users_for_role(role_id, role_user_search_query_request)
Search role users

Search users with assigned role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**role_user_search_query_request** | Option<[**RoleUserSearchQueryRequest**](RoleUserSearchQueryRequest.md)> |  |  |

### Return type

[**models::RoleUserSearchResult**](RoleUserSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_role_from_client

> unassign_role_from_client(role_id, client_id)
Unassign a role from a client

Unassigns the specified role from the client. The client will no longer inherit the authorizations associated with this role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**client_id** | **String** | The client ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_role_from_group

> unassign_role_from_group(role_id, group_id)
Unassign a role from a group

Unassigns the specified role from the group. All group members (user or client) no longer inherit the authorizations associated with this role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**group_id** | **String** | The group ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_role_from_mapping_rule

> unassign_role_from_mapping_rule(role_id, mapping_rule_id)
Unassign a role from a mapping rule

Unassigns a role from a mapping rule.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**mapping_rule_id** | **String** | The mapping rule ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_role_from_user

> unassign_role_from_user(role_id, username)
Unassign a role from a user

Unassigns a role from a user. The user will no longer inherit the authorizations associated with this role.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**username** | **String** | The user username. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_role

> models::RoleUpdateResult update_role(role_id, role_update_request)
Update role

Update a role with the given ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_id** | **String** | The role ID. | [required] |
**role_update_request** | [**RoleUpdateRequest**](RoleUpdateRequest.md) |  | [required] |

### Return type

[**models::RoleUpdateResult**](RoleUpdateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

