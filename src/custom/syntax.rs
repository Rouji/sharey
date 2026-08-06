use core::{
    iter::Iterator,
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
    vec.iter().cloned().collect::<String>()
}

fn string_to_char_vec(string: &str) -> Vec<char> {
    string.chars().collect()
}

fn process_int<F, D, E>(
    input: &[char],
    is_func: bool,
    start: usize,
    func_callback: &F,
    user_data: &D,
) -> Result<(Vec<char>, usize), E>
where
    F: Fn(&str, &[String], &D) -> Result<String, E>,
{
    let mut escape = false;
    let mut out: Vec<char> = Vec::new();
    let mut i = start;
    'outer: while i < input.len() {
        let c = input[i];
        if !escape {
            match c {
                FUNC_START => {
                    let (processed, j) = process_int(input, true, i + 1, func_callback, user_data)?;
                    out.extend(processed);
                    i = j + 1;
                    continue;
                }
                FUNC_END | ARG_DELIM => {
                    break 'outer;
                }
                ESCAPE => {
                    escape = true;
                    i += 1;
                    continue;
                }
                ARG_START if is_func => {
                    let mut params: Vec<String> = Vec::new();
                    'inner: loop {
                        let (processed, j) =
                            process_int(input, false, i + 1, func_callback, user_data)?;
                        i = j;
                        params.push(char_vec_to_string(&processed));
                        if !(i < input.len() && input[i] == ARG_DELIM) {
                            break 'inner;
                        }
                    }
                    let call_ret = func_callback(&char_vec_to_string(&out), &params, user_data)?;
                    return Ok((string_to_char_vec(&call_ret), i));
                }
                _ => (),
            }
        }

        escape = false;
        out.push(c);
        i += 1;
    }

    if is_func {
        let call_ret = func_callback(&char_vec_to_string(&out), &Vec::new(), user_data)?;
        return Ok((string_to_char_vec(&call_ret), i));
    }
    Ok((out, i))
}

pub fn process<F, D, E>(input: &str, func_callback: &F, user_data: &D) -> Result<String, E>
where
    F: Fn(&str, &[String], &D) -> Result<String, E>,
{
    let input_chars = if is_probably_old_syntax(input) {
        string_to_char_vec(&upgrade_old_syntax(input))
    } else {
        string_to_char_vec(input)
    };
    let (output, _j) = process_int(&input_chars, false, 0, func_callback, user_data)?;
    Ok(char_vec_to_string(&output))
}

pub fn is_probably_old_syntax(input: &str) -> bool {
    PROBABLY_OLD_SYNTAX_RE.is_match(input)
}

pub fn upgrade_old_syntax(input: &str) -> String {
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
