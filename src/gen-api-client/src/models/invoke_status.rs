/*
 * JJS main API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvokeStatus {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "kind")]
    pub kind: String,
}

impl InvokeStatus {
    pub fn new(code: String, kind: String) -> InvokeStatus {
        InvokeStatus { code, kind }
    }
}