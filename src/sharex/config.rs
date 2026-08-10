use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use thiserror;

fn default_request_method() -> RequestMethod {
    RequestMethod::Post
}
fn default_file_form_name() -> String {
    "file".to_string()
}
fn default_response_type() -> String {
    "Text".to_string()
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum RequestMethod {
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "PUT")]
    Put,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "DELETE")]
    Delete,
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum DestinationType {
    ImageUploader,
    TextUploader,
    FileUploader,
    #[serde(alias = "URLSharingService")]
    URLShortener,
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum Body {
    MultipartFormData,
    #[serde(rename = "JSON")]
    Json,
    #[serde(rename = "XML")]
    Xml,
    Binary,
    FormURLEncoded,
    None,
}

fn deserialize_destination_types<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<DestinationType>>, D::Error>
where
    D: Deserializer<'de>,
{
    let comma_separated: &str = Deserialize::deserialize(deserializer)?;
    let types = comma_separated
        .split(',')
        .map(|piece| DestinationType::deserialize(piece.trim().into_deserializer()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(types))
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Config {
    pub version: Option<String>,
    pub name: Option<String>,
    pub regex_list: Option<Vec<String>>,
    pub body: Option<Body>,
    pub arguments: Option<HashMap<String, Option<String>>>, // goes into the body?
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

impl Config {
    pub fn from_string(string: &str) -> Result<Config, Error> {
        Ok(serde_json::from_str::<Self>(string)?)
    }

    pub fn from_file(path: &str) -> Result<Config, Error> {
        let f = std::fs::File::open(path)?;
        let s = std::io::read_to_string(f)?;
        Self::from_string(&s)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O Error")]
    IOError(#[from] std::io::Error),
    #[error("Serialisation Error")]
    SerialisationError(#[from] serde_json::Error),
}
