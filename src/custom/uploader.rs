use crate::custom::config::{Body, Config, RequestMethod};
use crate::custom::syntax::process;
use base64;
use mime_guess;
use rand;
use reqwest;
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};
use thiserror;

struct SyntaxFuncData<'a> {
    config: &'a Config,
    response_text: Option<String>,
    response_headers: HashMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid ShareX syntax")]
    Syntax(String),
    #[error("Request error")]
    Request(#[from] reqwest::Error),
}

fn syntax_func_callback(
    name: &String,
    args: &Vec<String>,
    data: &SyntaxFuncData,
) -> Result<String, Error> {
    match name.as_str() {
        "response" => match &data.response_text {
            Some(text) => Ok(text.to_string()),
            None => Err(Error::Syntax(
                "{response} is not available in current context".to_string(),
            )),
        },
        "base64" => match args.get(0) {
            Some(arg) => Ok(base64::encode(arg.as_bytes())),
            None => Err(Error::Syntax("base64 needs exactly 1 argument".to_string())),
        },
        "random" => {
            if args.len() < 1 {
                Err(Error::Syntax(
                    "{random} needs at least 1 argument".to_string(),
                ))
            } else {
                let rnd_i = rand::random::<usize>() % args.len();
                Ok(args[rnd_i].to_string())
            }
        }
        "header" => {
            match args.get(0) {
                Some(name) => Ok(match data.response_headers.get(name) {
                    Some(val) => val.to_string(),
                    None => "".to_string(),
                }),
                None => Err(Error::Syntax("{header} needs 1 argument".to_string())),
            }
        }
        //TODO
        "select" | "prompt" => Err(Error::Syntax(format!("unsupported function {{{}}}", name))),
        _ => Err(Error::Syntax(format!("invalid function {{{}}}", name))),
    }
}

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
        let mime_type = mime_guess::from_path(file_path).first_raw();
        let reader = std::io::BufReader::new(f);
        Ok(Self {
            reader: Some(Box::new(reader)),
            content_size,
            mime_type,
            file_name,
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

impl CustomUploader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn upload(&self, input: Input) -> Result<Output, Error> {
        let c = reqwest::blocking::Client::new();

        let mut req = match self.config.request_method {
            RequestMethod::GET => c.request(reqwest::Method::GET, &self.config.request_url),
            RequestMethod::POST => c.request(reqwest::Method::POST, &self.config.request_url),
            RequestMethod::PUT => c.request(reqwest::Method::PUT, &self.config.request_url),
            RequestMethod::PATCH => c.request(reqwest::Method::PATCH, &self.config.request_url),
            RequestMethod::DELETE => c.request(reqwest::Method::DELETE, &self.config.request_url),
        };

        let mut syntax_func_data = SyntaxFuncData {
            config: &self.config,
            response_text: None,
            response_headers: HashMap::new(),
        };
        let syn = |s| process(s, syntax_func_callback, &syntax_func_data);

        if let Some(h) = &self.config.headers {
            let mut header_map = reqwest::header::HeaderMap::new();
            for (k, v) in h.iter() {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&syn(v)?),
                ) {
                    header_map.insert(name, val);
                }
            }
            req = req.headers(header_map);
        }

        if let Some(param) = &self.config.parameters {
            let mut param_map: HashMap<String, String> = HashMap::new();
            for (k, v) in param.iter() {
                param_map.insert(k.to_string(), syn(v)?);
            }
            req = req.query(&param_map);
        }

        req = match self.config.body {
            //Some(Body::OnceToldMe)
            Some(Body::MultipartFormData) => {
                let mut form = reqwest::blocking::multipart::Form::new();
                if let Some(args) = &self.config.arguments {
                    for (k, v) in args {
                        form = form.text(k.clone(), syn(v)?);
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
            Some(Body::FormURLEncoded) => {
                if input.reader.is_some() {
                    unimplemented!("sending FormURLEncoded with data (a reader) isn't supported");
                }
                req.form(&self.config.parameters)
            }
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

        let res = req.send()?.error_for_status()?;

        for (k, v) in res.headers() {
            if let Ok(val) = v.to_str() {
                syntax_func_data.response_headers.insert(k.to_string(), val.to_string());
            }
        }

        let res_text = res.text()?.clone();
        syntax_func_data.response_text = Some(res_text.to_string().clone());

        let syn = |s| process(s, syntax_func_callback, &syntax_func_data);


        Ok(Output {
            response: Some(res_text.clone()),
            url: Some(match &self.config.url {
                Some(url) => syn(&url)?,
                None => res_text,
            }),
            deletion_url: match &self.config.deletion_url {
                Some(url) => Some(syn(&url)?),
                None => None,
            },
            thumbnail_url: match &self.config.thumbnail_url {
                Some(url) => Some(syn(&url)?),
                None => None,
            },
            error_message: match &self.config.error_message {
                Some(err) => Some(syn(&err)?),
                None => None,
            },
        })
    }
}
