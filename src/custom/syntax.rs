use core::{
    iter::{Iterator, Peekable},
    result::Result::{self},
};
use regex::Regex;

const FUNC_START: char = '{';
const FUNC_END: char = '}';
const ARG_START: char = ':';
const ARG_DELIM: char = '|';
const ESCAPE: char = '\\';

static PROBABLY_OLD_SYNTAX_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"((^|[^\\])\$.*){2}").unwrap());

fn char_vec_to_string(vec: &[char]) -> String {
    vec.iter().collect::<String>()
}

fn string_to_char_vec(string: &str) -> Vec<char> {
    string.chars().collect()
}

fn process_int<F, D, E>(
    input: &mut Peekable<impl Iterator<Item = char>>,
    is_func: bool,
    func_callback: &F,
    user_data: &D,
) -> Result<(String, Option<char>), E>
where
    F: Fn(&str, &[String], &D) -> Result<String, E>,
{
    let mut escape = false;
    let mut out = String::with_capacity(20);
    let mut last_c: Option<char> = None;
    'outer: while let Some(c) = input.next() {
        if !escape {
            match c {
                FUNC_START => {
                    let (processed, _) = process_int(input, true, func_callback, user_data)?;
                    out += &processed;
                    continue;
                }
                FUNC_END | ARG_DELIM => {
                    last_c = Some(c);
                    break 'outer;
                }
                ESCAPE => {
                    escape = true;
                    continue;
                }
                ARG_START if is_func => {
                    let mut params: Vec<String> = Vec::new();
                    'inner: loop {
                        let (processed, last_c) =
                            process_int(input, false, func_callback, user_data)?;
                        params.push(processed);
                        if !(last_c == Some(ARG_DELIM)) {
                            break 'inner;
                        }
                    }
                    let call_ret = func_callback(&out, &params, user_data)?;
                    return Ok((call_ret, Some(c)));
                }
                _ => (),
            }
        }

        escape = false;
        out.push(c);
    }

    if is_func {
        let call_ret = func_callback(&out, &Vec::new(), user_data)?;
        return Ok((call_ret, last_c));
    }
    Ok((out, last_c))
}

pub fn process<F, D, E>(input: &str, func_callback: &F, user_data: &D) -> Result<String, E>
where
    F: Fn(&str, &[String], &D) -> Result<String, E>,
{
    let owned;
    let input_str = if is_probably_old_syntax(input) {
        owned = upgrade_old_syntax(input);
        &owned
    } else {
        input
    };
    let mut input_chars = input_str.chars().peekable();
    let (output, _) = process_int(&mut input_chars, false, func_callback, user_data)?;
    Ok(output)
}

fn is_probably_old_syntax(input: &str) -> bool {
    PROBABLY_OLD_SYNTAX_RE.is_match(input)
}

fn upgrade_old_syntax(input: &str) -> String {
    let mut output: String = String::with_capacity(input.len());
    let mut in_func: bool = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => output.push_str("\\{"),
            '}' => output.push_str("\\}"),
            '$' => output.push(match in_func {
                false => {
                    in_func = true;
                    '{'
                }
                true => {
                    in_func = false;
                    '}'
                }
            }),
            '\\' => output.push(match chars.peek() {
                Some('$') => {
                    chars.next();
                    '$'
                }
                _ => c,
            }),
            _ => output.push(c),
        }
    }
    output
}

#[cfg(test)]
mod test {
    use crate::custom::syntax::{is_probably_old_syntax, upgrade_old_syntax};

    #[test]
    fn old_syntax_re() {
        assert!(is_probably_old_syntax("$json:data.hash$"));
        assert!(is_probably_old_syntax("a$func$"));
        assert!(is_probably_old_syntax("one $func$ and another $func$"));
        assert!(!is_probably_old_syntax("\\$escaped$one"));
        assert!(!is_probably_old_syntax("only $ one dollar"));
        assert!(!is_probably_old_syntax("\\$\\$two escaped"));
    }

    #[test]
    fn upgrade() {
        assert_eq!(upgrade_old_syntax("$json:data.hash$"), "{json:data.hash}");
        assert_eq!(upgrade_old_syntax("a$func$"), "a{func}");
        assert_eq!(
            upgrade_old_syntax("one $func$ and another $func$"),
            "one {func} and another {func}"
        );
        assert_eq!(upgrade_old_syntax("\\$\\$two escaped"), "$$two escaped");
        assert_eq!(upgrade_old_syntax("$func:with|args$"), "{func:with|args}");
        assert_eq!(
            upgrade_old_syntax("{literalbraces}$func$"),
            "\\{literalbraces\\}{func}"
        );
    }
}
