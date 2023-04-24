const FUNCTION_START: char = '{';
const FUNCTION_END: char = '}';
const ARGUMENTS_START: char = ':';
const ARGUMENTS_DELIMITER: char = '|';
const ESCAPE: char = '\\';

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum Element {
    String(String),
    Function(Function),
}

fn char_vec_to_string(vec: &Vec<char>) -> String {
    vec.iter().cloned().collect::<String>()
}

fn parse(string: &str) -> Vec<Element> {
    let mut i: usize = 0;
    let input: Vec<char> = string.chars().collect();
    let mut elements: Vec<Element> = Vec::new();
    let mut function_name: Vec<char> = Vec::new();
    let mut function_args: Vec<Vec<char>> = Vec::new();
    let mut tmp: Vec<char> = Vec::new();
    let mut in_function = false;
    while let Some(c) = input.get(i) {
        match *c {
            ESCAPE => {
                if let Some(next) = input.get(i + 1) {
                    tmp.push(*next);
                }
                i += 2;
            }
            FUNCTION_START => {
                if tmp.len() > 0 {
                    elements.push(Element::String(char_vec_to_string(&tmp)));
                    tmp = Vec::new();
                }
                in_function = true;
                i += 1;
            }
            ARGUMENTS_START => {
                // allow things like {base64:user:pw} without escaping
                if function_name.len() > 0 || !in_function {
                    tmp.push(*c);
                }
                else {
                    function_name = tmp;
                    tmp = Vec::new();
                }
                i += 1;
            }
            ARGUMENTS_DELIMITER => {
                function_args.push(tmp);
                tmp = Vec::new();
                i += 1;
            }
            FUNCTION_END => {
                if function_name.len() == 0 {
                    function_name = tmp;
                    tmp = Vec::new();
                } else if tmp.len() > 0 {
                    function_args.push(tmp);
                    tmp = Vec::new();
                }
                elements.push(Element::Function(Function {
                    name: char_vec_to_string(&function_name),
                    args: function_args
                        .iter()
                        .map(|arg| char_vec_to_string(arg))
                        .collect(),
                }));
                function_args = Vec::new();
                function_name = Vec::new();
                in_function = false;
                i += 1;
            }
            _ => {
                tmp.push(*c);
                i += 1;
            }
        }
    }
    if tmp.len() > 0 {
        elements.push(Element::String(char_vec_to_string(&tmp)));
    }
    elements
}

pub fn process<F, D, E>(input: &str, func_callback: F, user_data: &D) -> Result<String, E>
where
    F: Fn(&String, &Vec<String>, &D) -> Result<String, E>,
{
    let mut output: String = String::new();
    for el in parse(input) {
        match el {
            Element::String(string) => output.push_str(&string),
            Element::Function(func_def) => {
                output.push_str(&func_callback(&func_def.name, &func_def.args, user_data)?)
            }
        }
    }
    Ok(output)
}
