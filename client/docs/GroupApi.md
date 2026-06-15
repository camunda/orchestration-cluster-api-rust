# \GroupApi

All URIs are relative to *http://localhost:8080/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assign_client_to_group**](GroupApi.md#assign_client_to_group) | **PUT** /groups/{groupId}/clients/{clientId} | Assign a client to a group
[**assign_mapping_rule_to_group**](GroupApi.md#assign_mapping_rule_to_group) | **PUT** /groups/{groupId}/mapping-rules/{mappingRuleId} | Assign a mapping rule to a group
[**assign_user_to_group**](GroupApi.md#assign_user_to_group) | **PUT** /groups/{groupId}/users/{username} | Assign a user to a group
[**create_group**](GroupApi.md#create_group) | **POST** /groups | Create group
[**delete_group**](GroupApi.md#delete_group) | **DELETE** /groups/{groupId} | Delete group
[**get_group**](GroupApi.md#get_group) | **GET** /groups/{groupId} | Get group
[**search_clients_for_group**](GroupApi.md#search_clients_for_group) | **POST** /groups/{groupId}/clients/search | Search group clients
[**search_groups**](GroupApi.md#search_groups) | **POST** /groups/search | Search groups
[**search_mapping_rules_for_group**](GroupApi.md#search_mapping_rules_for_group) | **POST** /groups/{groupId}/mapping-rules/search | Search group mapping rules
[**search_roles_for_group**](GroupApi.md#search_roles_for_group) | **POST** /groups/{groupId}/roles/search | Search group roles
[**search_users_for_group**](GroupApi.md#search_users_for_group) | **POST** /groups/{groupId}/users/search | Search group users
[**unassign_client_from_group**](GroupApi.md#unassign_client_from_group) | **DELETE** /groups/{groupId}/clients/{clientId} | Unassign a client from a group
[**unassign_mapping_rule_from_group**](GroupApi.md#unassign_mapping_rule_from_group) | **DELETE** /groups/{groupId}/mapping-rules/{mappingRuleId} | Unassign a mapping rule from a group
[**unassign_user_from_group**](GroupApi.md#unassign_user_from_group) | **DELETE** /groups/{groupId}/users/{username} | Unassign a user from a group
[**update_group**](GroupApi.md#update_group) | **PUT** /groups/{groupId} | Update group



## assign_client_to_group

> assign_client_to_group(group_id, client_id)
Assign a client to a group

Assigns a client to a group, making it a member of the group. Members of the group inherit the group authorizations, roles, and tenant assignments. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**client_id** | **String** | The client ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_mapping_rule_to_group

> assign_mapping_rule_to_group(group_id, mapping_rule_id)
Assign a mapping rule to a group

Assigns a mapping rule to a group.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**mapping_rule_id** | **String** | The mapping rule ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_user_to_group

> assign_user_to_group(group_id, username)
Assign a user to a group

Assigns a user to a group, making the user a member of the group. Group members inherit the group authorizations, roles, and tenant assignments. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**username** | **String** | The user username. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_group

> models::GroupCreateResult create_group(group_create_request)
Create group

Create a new group.  The supplied `groupId` is validated against `^[a-zA-Z0-9_~@.+-]+$` (max 256 characters) by `IdentifierValidator.validateId` in the runtime. This strict validation applies wherever the Groups API is available: in OIDC deployments that set `camunda.security.authentication.oidc.groupsClaim` the Groups API (including this endpoint) is disabled entirely, so group CRUD never sees externally-minted IdP IDs. The BYOG relaxation only loosens validation when a group is referenced *as a member* of a role or tenant (`assignRoleToGroup`, `assignGroupToTenant`); group CRUD itself always uses the strict default-id regex. The constraint is not advertised on the `GroupId` schema so that the same schema can be reused at member-reference sites without falsely rejecting externally-minted IdP group IDs there. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_create_request** | Option<[**GroupCreateRequest**](GroupCreateRequest.md)> |  |  |

### Return type

[**models::GroupCreateResult**](GroupCreateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_group

> delete_group(group_id)
Delete group

Deletes the group with the given ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_group

> models::GroupResult get_group(group_id)
Get group

Get a group by its ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |

### Return type

[**models::GroupResult**](GroupResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_clients_for_group

> models::GroupClientSearchResult search_clients_for_group(group_id, group_client_search_query_request)
Search group clients

Search clients assigned to a group.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**group_client_search_query_request** | Option<[**GroupClientSearchQueryRequest**](GroupClientSearchQueryRequest.md)> |  |  |

### Return type

[**models::GroupClientSearchResult**](GroupClientSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_groups

> models::GroupSearchQueryResult search_groups(group_search_query_request)
Search groups

Search for groups based on given criteria.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_search_query_request** | Option<[**GroupSearchQueryRequest**](GroupSearchQueryRequest.md)> |  |  |

### Return type

[**models::GroupSearchQueryResult**](GroupSearchQueryResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_mapping_rules_for_group

> models::GroupMappingRuleSearchResult search_mapping_rules_for_group(group_id, mapping_rule_search_query_request)
Search group mapping rules

Search mapping rules assigned to a group.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**mapping_rule_search_query_request** | Option<[**MappingRuleSearchQueryRequest**](MappingRuleSearchQueryRequest.md)> |  |  |

### Return type

[**models::GroupMappingRuleSearchResult**](GroupMappingRuleSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_roles_for_group

> models::GroupRoleSearchResult search_roles_for_group(group_id, role_search_query_request)
Search group roles

Search roles assigned to a group.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**role_search_query_request** | Option<[**RoleSearchQueryRequest**](RoleSearchQueryRequest.md)> |  |  |

### Return type

[**models::GroupRoleSearchResult**](GroupRoleSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_users_for_group

> models::GroupUserSearchResult search_users_for_group(group_id, group_user_search_query_request)
Search group users

Search users assigned to a group.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**group_user_search_query_request** | Option<[**GroupUserSearchQueryRequest**](GroupUserSearchQueryRequest.md)> |  |  |

### Return type

[**models::GroupUserSearchResult**](GroupUserSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_client_from_group

> unassign_client_from_group(group_id, client_id)
Unassign a client from a group

Unassigns a client from a group. The client is removed as a group member, with associated authorizations, roles, and tenant assignments no longer applied. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**client_id** | **String** | The client ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_mapping_rule_from_group

> unassign_mapping_rule_from_group(group_id, mapping_rule_id)
Unassign a mapping rule from a group

Unassigns a mapping rule from a group.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**mapping_rule_id** | **String** | The mapping rule ID. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_user_from_group

> unassign_user_from_group(group_id, username)
Unassign a user from a group

Unassigns a user from a group. The user is removed as a group member, with associated authorizations, roles, and tenant assignments no longer applied. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**username** | **String** | The user username. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_group

> models::GroupUpdateResult update_group(group_id, group_update_request)
Update group

Update a group with the given ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The group ID. | [required] |
**group_update_request** | [**GroupUpdateRequest**](GroupUpdateRequest.md) |  | [required] |

### Return type

[**models::GroupUpdateResult**](GroupUpdateResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

