const FUNCTION_START: char = '{';
const FUNCTION_END: char = '}';
const ARGUMENTS_START: char = ':';
const ARGUMENTS_DELIMITER: char = '|';
const ESCAPE: char = '\\';

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Vec<String>,
}

#[derive(Debug)]
pub enum Element {
    String(String),
    Function(Function),
}

fn char_vec_to_string(vec: &Vec<char>) -> String {
    vec.iter().cloned().collect::<String>()
}

pub fn parse(string: &str) -> Vec<Element> {
    println!("{}", string);
    let mut i: usize = 0;
    let input: Vec<char> = string.chars().collect();
    let mut elements: Vec<Element> = Vec::new();
    let mut function_name: Vec<char> = Vec::new();
    let mut function_args: Vec<Vec<char>> = Vec::new();
    let mut tmp: Vec<char> = Vec::new();
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
                i += 1;
            }
            ARGUMENTS_START => {
                function_name = tmp;
                tmp = Vec::new();
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

