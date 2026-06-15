# LicenseResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**valid_license** | **bool** | True if the Camunda license is valid, false if otherwise | 
**license_type** | **String** | Will return the license type property of the Camunda license | 
**is_commercial** | **bool** | Will be false when a license contains a non-commerical=true property | 
**expires_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The date when the Camunda license expires | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


