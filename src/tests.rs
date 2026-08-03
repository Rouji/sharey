#![cfg(test)]
use core::result::Result::Ok;

use crate::custom::syntax::process;

struct SyntaxFuncData {}
struct Error {}

static EMPTY_DATA: SyntaxFuncData = SyntaxFuncData {};

fn cb(name: &str, args: &[String], _data: &SyntaxFuncData) -> Result<String, Error> {
    match name {
        "hello" => Ok("Hello".to_string()),
        "onearg" => match args.len() {
            1 => Ok("".to_string()),
            _ => Err(Error {}),
        },
        "twoarg" => match args.len() {
            2 => Ok("".to_string()),
            _ => Err(Error {}),
        },
        "echo" => match args.len() {
            1 => Ok(args[0].clone()),
            _ => Err(Error {}),
        },
        _ => Err(Error {}),
    }
}

macro_rules! parser_test {
    ($func_name:ident, $input:literal, $output:literal) => {
        #[test]
        fn $func_name() {
            let res = process($input, &cb, &EMPTY_DATA);
            assert!(res.is_ok());
            if let Ok(str) = res {
                assert_eq!(str, $output);
            }
        }
    };
}

macro_rules! parser_test_failing {
    ($func_name:ident, $input:literal) => {
        #[test]
        fn $func_name() {
            let res = process($input, &cb, &EMPTY_DATA);
            assert!(!res.is_ok());
        }
    };
}

parser_test_failing!(nonexistent_func, "{doesntexist}");

parser_test_failing!(missing_arg, "{onearg}");
parser_test_failing!(too_many_args, "{onearg:one|two}");
parser_test!(one_arg, "{onearg:one}", "");
parser_test!(two_arg, "{twoarg:one|two}", "");
parser_test_failing!(too_few_args, "{twoarg:one}");

parser_test!(simple_func_call, "{hello}", "Hello");
parser_test!(
    escaped_func_call_truncates,
    "\\{doesntexist}blah",
    "{doesntexist"
); //sharex parser does this
parser_test!(single_closing_bracket_truncates, "some}text", "some"); //sharex parser does this
parser_test!(
    no_arg_func_with_output,
    "blah {hello} blub",
    "blah Hello blub"
);
parser_test!(single_escaped_opening_bracket, "a\\{b", "a{b");
parser_test!(two_func_calls, "{hello}{hello}", "HelloHello");

parser_test!(nested_call, "{echo:{hello}}", "Hello");
