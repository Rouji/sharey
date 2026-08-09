// TODO: remove once fields/functions below are actually wired up
#![allow(dead_code)]

use std::env;

mod sharex;
use sharex::config::Config;
use sharex::uploader::{Input, SharexUploader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config_path = &args[1];
    let file_path = &args[2];

    let _syntext = "{a}hello {test:123}{rand\\\\om}{test2:more|ar\\}gs} world";

    let config = Config::from_file(config_path)?;

    let uploader = SharexUploader::new(config);

    let input = Input::from_file(file_path)?;

    let _ = dbg!(uploader.upload(input));
    Ok(())
}
