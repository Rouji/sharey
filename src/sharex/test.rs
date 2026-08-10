use crate::sharex::config;

macro_rules! sxcu_test {
    ($func_name:ident, $path:literal) => {
        #[test]
        #[allow(non_snake_case)]
        fn $func_name() {
            let loaded_config = config::Config::from_file($path);
            assert!(!loaded_config.is_err());
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/custom_uploaders_tests.rs"));
