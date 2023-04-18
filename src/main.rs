use std::{env, fs};

mod custom;
use custom::config::CustomUploaderConfig;
use custom::uploader::{CustomUploader,Input};



fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = &args[1];
    let file_path = &args[2];

    let json = fs::read_to_string(config_path).expect("can't read that file m8");

    let parsed: CustomUploaderConfig = serde_json::from_str(&json).unwrap();

    let uploader = CustomUploader::new(parsed);

    let input = Input::from_file(file_path).unwrap();

    if let Ok(res) = uploader.upload(input) {
        print!("{}", res);
    }
}
