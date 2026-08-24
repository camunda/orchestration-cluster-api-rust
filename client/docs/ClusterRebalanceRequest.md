# ClusterRebalanceRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**replication_lag_threshold** | Option<**i64**> | The highest replication lag (in bytes) that a desired leader may have for its transfer to be accepted. | [optional]
**replication_timeout** | Option<**String**> | How long a partition may stay frozen waiting for its desired leader to catch up (as a positive ISO-8601 duration). | [optional]
**max_transfer_attempts** | Option<**i32**> | How many times a current leader may prompt the desired leader to take over leadership before giving up. | [optional]
**leader_wait_timeout** | Option<**String**> | How long the coordinator waits for a partition without a leader to acquire one before reporting `NO_LEADER` and moving on (as a positive ISO-8601 duration). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


