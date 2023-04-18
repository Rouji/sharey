use serde::{Deserialize, Deserializer};
use serde_json;
use std::collections::HashMap;

fn default_request_method() -> RequestMethod {
    RequestMethod::POST
}
fn default_file_form_name() -> String {
    "file".to_string()
}
fn default_response_type() -> String {
    "Text".to_string()
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum RequestMethod {
    POST,
    GET,
    PUT,
    PATCH,
    DELETE,
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum DestinationType {
    ImageUploader,
    TextUploader,
    FileUploader,
    URLShortener,
    URLSharingService,
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum Body {
    MultipartFormData,
    JSON,
    XML,
    Binary,
    FormURLEncoded,
    None,
}

//format comma-separated list (with possibly spaces in there) into a proper json list of strings
//and parse that instead
fn deserialize_destination_types<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<DestinationType>>, D::Error>
where
    D: Deserializer<'de>,
{
    let comma_separated: &str = Deserialize::deserialize(deserializer)?;
    let json_list = format!(
        "[{}]",
        comma_separated
            .split(",")
            .map(|p| format!("\"{}\"", p.trim()))
            .collect::<Vec<String>>()
            .join(",")
    );
    Ok(Some(serde_json::from_str(&json_list).unwrap()))
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct CustomUploaderConfig {
    pub version: Option<String>,
    pub name: Option<String>,
    pub regex_list: Option<Vec<String>>,
    pub body: Option<Body>,
    pub arguments: Option<HashMap<String, String>>, // goes into the body?
    pub headers: Option<HashMap<String, String>>,
    pub parameters: Option<HashMap<String, String>>, // query string
    pub data: Option<String>,
    pub error_message: Option<String>,
    #[serde(default, deserialize_with = "deserialize_destination_types")]
    pub destination_type: Option<Vec<DestinationType>>,
    #[serde(default = "default_file_form_name")]
    pub file_form_name: String,
    #[serde(default = "default_response_type")]
    pub response_type: String,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    #[serde(rename = "ThumbnailURL")]
    pub thumbnail_url: Option<String>,
    #[serde(rename = "DeletionURL")]
    pub deletion_url: Option<String>,
    #[serde(alias = "RequestType", default = "default_request_method")]
    pub request_method: RequestMethod,
    #[serde(rename = "RequestURL")]
    pub request_url: String,
}
