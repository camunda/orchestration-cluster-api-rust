# SetVariableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**variables** | **std::collections::HashMap<String, serde_json::Value>** | JSON object representing the variables to set in the element’s scope. | 
**local** | Option<**bool**> | If set to `true`, the variables are merged strictly into the local scope (as specified by the `elementInstanceKey`). Otherwise, the variables are propagated to upper scopes and set at the outermost one.  Let's consider the following example: There are two scopes '1' and '2'. Scope '1' is the parent scope of '2'. The effective variables of the scopes are: 1 => { \"foo\" : 2 } 2 => { \"bar\" : 1 }  An update request with elementInstanceKey as '2', variables { \"foo\": 5 }, and local set to `true` leaves scope '1' unchanged and adjusts scope '2' to { \"bar\": 1, \"foo\": 5 }. By default, with local set to `false`, scope '1' will be { \"foo\": 5 } and scope '2' will be { \"bar\": 1 }. | [optional][default to false]
**operation_reference** | Option<**i64**> | A reference key chosen by the user that will be part of all records resulting from this operation. Must be > 0 if provided.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


