const FUNC_START: char = '{';
const FUNC_END: char = '}';
const ARG_START: char = ':';
const ARG_DELIM: char = '|';
const ESCAPE: char = '\\';

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
                        println!(
                            "{}-{}: {} -> {}",
                            i + 1,
                            j,
                            char_vec_to_string(&input[i + 1..j]),
                            char_vec_to_string(&processed)
                        );
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
    let input_chars = string_to_char_vec(input);
    let (output, _j) = process_int(&input_chars, false, 0, func_callback, user_data)?;
    Ok(char_vec_to_string(&output))
}
