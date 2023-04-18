use crate::custom::config::{Config, Body, RequestMethod};
use reqwest;
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};

pub struct CustomUploader {
    config: Config,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Input<'a> {
    reader: Option<Box<dyn std::io::Read + std::marker::Send>>,
    content_size: Option<u64>,
    file_name: Option<&'a str>,
    mime_type: Option<&'a str>,
}

impl<'a> Input<'a> {
    pub fn from_file(file_path: &'a str) -> Result<Self, std::io::Error> {
        let mut f = std::fs::File::open(file_path)?;
        let file_name = match std::path::Path::new(file_path).file_name() {
            Some(name) => name.to_str(),
            None => Some("file.bin"),
        };
        let content_size = match f.seek(SeekFrom::End(0)) {
            Ok(size) => {
                f.seek(SeekFrom::Start(0))?;
                Some(size)
            }
            Err(_) => None,
        };
        let reader = std::io::BufReader::new(f);
        Ok(Self {
            reader: Some(Box::new(reader)),
            content_size,
            file_name,
            mime_type: Some("application/octet-stream"),
        })
    }
}

impl CustomUploader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn upload(&self, input: Input) -> Result<String, Box<dyn std::error::Error>> {
        let c = reqwest::blocking::Client::new();

        let mut req = match self.config.request_method {
            RequestMethod::GET => c.request(reqwest::Method::GET, &self.config.request_url),
            RequestMethod::POST => c.request(reqwest::Method::POST, &self.config.request_url),
            RequestMethod::PUT => c.request(reqwest::Method::PUT, &self.config.request_url),
            RequestMethod::PATCH => c.request(reqwest::Method::PATCH, &self.config.request_url),
            RequestMethod::DELETE => c.request(reqwest::Method::DELETE, &self.config.request_url),
        };

        let mut header_map = reqwest::header::HeaderMap::new();
        if let Some(h) = &self.config.headers {
            for (k, v) in h.iter() {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(v),
                ) {
                    header_map.insert(name, val);
                }
            }
        }
        req = req.headers(header_map);

        req = req.query(&self.config.parameters);

        req = match self.config.body {
            //Some(Body::OnceToldMe)
            Some(Body::MultipartFormData) => {
                let mut form = reqwest::blocking::multipart::Form::new();
                if let Some(args) = &self.config.arguments {
                    for (k, v) in args {
                        form = form.text(k.clone(), v.clone());
                    }
                }

                if let Some(reader) = input.reader {
                    let file_part = match input.content_size {
                        Some(size) => {
                            reqwest::blocking::multipart::Part::reader_with_length(reader, size)
                        }
                        None => reqwest::blocking::multipart::Part::reader(reader),
                    }
                    .mime_str(match input.mime_type {
                        Some(mime) => mime,
                        None => "application/octet-stream",
                    })?
                    .file_name(match input.file_name {
                        Some(name) => name.to_string(),
                        None => "file.bin".to_string(),
                    });

                    form = form.part(self.config.file_form_name.clone(), file_part);
                } else {
                    unimplemented!("sending multipart without data (a reader) isn't supported");
                }
                req.multipart(form)
            }
            Some(Body::FormURLEncoded) => req.form(&self.config.parameters),
            Some(Body::JSON) => {
                if let Some(json) = &self.config.arguments {
                    if let Ok(string) = serde_json::to_string(&json) {
                        req.header("Content-Type", "application/json").body(string)
                    } else {
                        req
                    }
                } else {
                    req
                }
            }
            Some(Body::None) | None => {
                let mut params = HashMap::new();
                if let Some(p) = &self.config.parameters {
                    params.extend(p);
                }
                if let Some(a) = &self.config.arguments {
                    params.extend(a);
                }
                req.query(&params)
            }
            Some(Body::XML) | Some(Body::Binary) => {
                unimplemented!();
            }
        };

        let res = req.send()?.error_for_status();

        Ok(res?.text()?)
    }
}
