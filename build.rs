fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::env::var("OUT_DIR").unwrap() + "/custom_uploaders_tests.rs";
    let custom_uploaders_path = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/CustomUploaders";
    let mut tests_string = String::new();
    for entry in std::fs::read_dir(custom_uploaders_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sxcu") {
            continue;
        }
        let stem = path.file_stem().and_then(|e| e.to_str()).unwrap();
        let test_name: String = stem
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect();

        let path_str = path.display();
        tests_string += &format!("sxcu_test!(test_{test_name}, \"{path_str}\");");
    }
    let _ = std::fs::write(out_path, tests_string);
    Ok(())
}
