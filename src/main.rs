use std::env;

mod custom;
use custom::config::Config;
use custom::uploader::{CustomUploader,Input};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config_path = &args[1];
    let file_path = &args[2];


    let config = Config::from_file(config_path)?;

    let uploader = CustomUploader::new(config);

    let input = Input::from_file(file_path).unwrap();

    if let Ok(res) = uploader.upload(input) {
        print!("{}", res);
    }
    Ok(())
}
