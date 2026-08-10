use crate::sharex::config::{Body, Config, RequestMethod};
use crate::sharex::template::process;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;
use core::option::Option;
use mime_guess;
use rand;
use reqwest;
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};
use thiserror;

struct SharexTemplateFuncData<'a> {
    config: &'a Config,
    response_text: Option<String>,
    response_headers: HashMap<String, String>,
    file_name: Option<String>,
    input: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid ShareX syntax")]
    SharexTemplate(String),
    #[error("Request error")]
    Request(#[from] reqwest::Error),
}

fn sharex_template_func_callback(
    name: &str,
    args: &[String],
    data: &SharexTemplateFuncData,
) -> Result<String, Error> {
    match name {
        "response" => match &data.response_text {
            Some(text) => Ok(text.to_string()),
            _ => Err(Error::SharexTemplate(
                "{response} is not available in current context".to_string(),
            )),
        },
        "base64" => match args.first() {
            Some(arg) => Ok(base64_engine.encode(arg.as_bytes())),
            _ => Err(Error::SharexTemplate(
                "base64 needs exactly 1 argument".to_string(),
            )),
        },
        "random" => {
            if args.is_empty() {
                Err(Error::SharexTemplate(
                    "{random} needs at least 1 argument".to_string(),
                ))
            } else {
                let rnd_i = rand::random::<usize>() % args.len();
                Ok(args[rnd_i].to_string())
            }
        }
        "header" => match args.first() {
            Some(name) => Ok(match data.response_headers.get(name) {
                Some(val) => val.to_string(),
                _ => "".to_string(),
            }),
            _ => Err(Error::SharexTemplate(
                "{header} needs 1 argument".to_string(),
            )),
        },
        "filename" => {
            if let Some(file_name) = &data.file_name {
                Ok(file_name.to_string())
            } else {
                Err(Error::SharexTemplate("file name unknown".to_string()))
            }
        }
        "input" => match &data.input {
            Some(input) => Ok(input.to_string()),
            _ => Err(Error::SharexTemplate("no text input".to_string())),
        },
        //TODO
        "select" | "prompt" | "json" | "xml" | "regex" => Err(Error::SharexTemplate(format!(
            "unsupported function {{{}}}",
            name
        ))),
        "dbg" => {
            dbg!(name);
            dbg!(args);
            Ok("dbg".to_string())
        }
        _ => Err(Error::SharexTemplate(format!(
            "invalid function {{{}}}",
            name
        ))),
    }
}

pub struct SharexUploader {
    config: Config,
}

pub struct Input<'a> {
    reader: Box<dyn std::io::Read + std::marker::Send>,
    content_size: Option<u64>,
    file_name: Option<&'a str>,
    mime_type: Option<&'a str>,
    text_input: Option<&'a str>,
}

impl<'a> Input<'a> {
    pub fn from_file(file_path: &'a str) -> Result<Self, std::io::Error> {
        let mut f = std::fs::File::open(file_path)?;
        let file_name = match std::path::Path::new(file_path).file_name() {
            Some(name) => name.to_str(),
            _ => Some("file.bin"),
        };
        let content_size = match f.seek(SeekFrom::End(0)) {
            Ok(size) => {
                f.seek(SeekFrom::Start(0))?;
                Some(size)
            }
            Err(_) => None,
        };
        let mime_type = mime_guess::from_path(file_path).first_raw();
        let reader = std::io::BufReader::new(f);
        Ok(Self {
            reader: Box::new(reader),
            content_size,
            mime_type,
            file_name,
            text_input: None,
        })
    }
}

#[derive(Debug)]
pub struct Output {
    pub response: Option<String>,
    pub url: Option<String>,
    pub deletion_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub error_message: Option<String>,
}

impl SharexUploader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn upload(&self, input: Input) -> Result<Output, Error> {
        let c = reqwest::blocking::Client::new();

        let mut req = c.request(
            match self.config.request_method {
                RequestMethod::Get => reqwest::Method::GET,
                RequestMethod::Post => reqwest::Method::POST,
                RequestMethod::Put => reqwest::Method::PUT,
                RequestMethod::Patch => reqwest::Method::PATCH,
                RequestMethod::Delete => reqwest::Method::DELETE,
            },
            &self.config.request_url,
        );

        let mut syntax_func_data = SharexTemplateFuncData {
            config: &self.config,
            response_text: None,
            response_headers: HashMap::new(),
            file_name: input.file_name.map(|file_name| file_name.to_string()),
            input: input.text_input.map(|input| input.to_string()),
        };
        let templ = |s| process(s, &sharex_template_func_callback, &syntax_func_data);

        if let Some(h) = &self.config.headers {
            let mut header_map = reqwest::header::HeaderMap::new();
            for (k, v) in h.iter() {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&templ(v)?),
                ) {
                    header_map.insert(name, val);
                }
            }
            req = req.headers(header_map);
        }

        if let Some(param) = &self.config.parameters {
            let mut param_map: HashMap<String, String> = HashMap::new();
            for (k, v) in param.iter() {
                param_map.insert(k.to_string(), templ(v)?);
            }
            req = req.query(&param_map);
        }

        req = match self.config.body {
            //Some(Body::OnceToldMe)
            Some(Body::MultipartFormData) => {
                let mut form = reqwest::blocking::multipart::Form::new();
                if let Some(args) = &self.config.arguments {
                    for (k, v) in args {
                        if let Some(val) = v {
                            form = form.text(k.clone(), templ(val)?);
                        }
                    }
                }

                let file_part = match input.content_size {
                    Some(size) => {
                        reqwest::blocking::multipart::Part::reader_with_length(input.reader, size)
                    }
                    _ => reqwest::blocking::multipart::Part::reader(input.reader),
                }
                .mime_str(match input.mime_type {
                    Some(mime) => mime,
                    _ => "application/octet-stream",
                })?
                .file_name(match input.file_name {
                    Some(name) => name.to_string(),
                    _ => "file.bin".to_string(),
                });

                form = form.part(self.config.file_form_name.clone(), file_part);
                req.multipart(form)
            }
            Some(Body::FormURLEncoded) => {
                unimplemented!("TODO");
            }
            Some(Body::Json) => {
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
                    params.extend(a.iter().filter_map(|(k, v)| v.as_ref().map(|v| (k, v))));
                }
                req.query(&params)
            }
            Some(Body::Xml) | Some(Body::Binary) => {
                unimplemented!();
            }
        };

        let res = req.send()?.error_for_status()?;

        for (k, v) in res.headers() {
            if let Ok(val) = v.to_str() {
                syntax_func_data
                    .response_headers
                    .insert(k.to_string(), val.to_string());
            }
        }

        let res_text = res.text()?.clone();
        syntax_func_data.response_text = Some(res_text.to_string().clone());

        let syn = |s| process(s, &sharex_template_func_callback, &syntax_func_data);

        Ok(Output {
            response: Some(res_text.clone()),
            url: Some(match &self.config.url {
                Some(url) => syn(url)?,
                _ => res_text,
            }),
            deletion_url: match &self.config.deletion_url {
                Some(url) => Some(syn(url)?),
                _ => None,
            },
            thumbnail_url: match &self.config.thumbnail_url {
                Some(url) => Some(syn(url)?),
                _ => None,
            },
            error_message: match &self.config.error_message {
                Some(err) => Some(syn(err)?),
                _ => None,
            },
        })
    }
}
