use crate::sharex::config;
use crate::sharex::template;

fn template_cb_stub(name: &str, _args: &[String], _data: &()) -> Result<String, ()> {
    match name {
        "response" | "base64" | "random" | "header" | "filename" | "input" | "select"
        | "prompt" | "json" | "xml" | "regex" | "dbg" | "responseurl" => Ok(String::new()),
        _ => Err(()),
    }
}
fn check_templating(config: &config::Config) {
    let mut strings: Vec<&String> = Vec::new();

    if let Some(headers) = &config.headers {
        strings.extend(headers.values());
    }
    if let Some(parameters) = &config.parameters {
        strings.extend(parameters.values());
    }
    if let Some(arguments) = &config.arguments {
        strings.extend(arguments.values().flatten()); // Option<String> values, drop the Nones
    }
    strings.extend(
        [
            config.url.as_ref(),
            config.thumbnail_url.as_ref(),
            config.deletion_url.as_ref(),
            config.error_message.as_ref(),
        ]
        .into_iter()
        .flatten(),
    );

    for s in strings {
        assert!(
            template::process(s, &template_cb_stub, &()).is_ok(),
            "templating failed for {s:?}"
        );
    }
}

macro_rules! sxcu_test {
    ($func_name:ident, $path:literal) => {
        #[test]
        #[allow(non_snake_case)]
        fn $func_name() {
            let loaded_config = config::Config::from_file($path);
            assert!(!loaded_config.is_err());
            check_templating(&loaded_config.unwrap());
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/custom_uploaders_tests.rs"));
