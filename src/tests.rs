use crate::custom::syntax::process;

#[cfg(test)]
mod tests {
    use crate::custom::syntax::process;

    struct SyntaxFuncData {}
    struct Error {}

    static EMPTY_DATA: SyntaxFuncData = SyntaxFuncData {};

    fn cb(name: &String, args: &Vec<String>, _data: &SyntaxFuncData) -> Result<String, Error> {
        match name.as_str() {
            "hello" => Ok("Hello".to_string()),
            "onearg" => match args.len() {
                1 => Ok("".to_string()),
                _ => Err(Error {}),
            },
            _ => Err(Error {}),
        }
    }

    #[test]
    fn noargs() {
        let res = process("{hello}", &cb, &EMPTY_DATA);
        assert!(res.is_ok());
        if let Ok(str) = res {
            assert_eq!(str, "Hello");
        }
    }

    #[test]
    fn onearg() {
        let res = process("{onearg}", &cb, &EMPTY_DATA);
        assert!(!res.is_ok());

        let res = process("{onearg:one|two}", &cb, &EMPTY_DATA);
        assert!(!res.is_ok());

        let res = process("{onearg:one}", &cb, &EMPTY_DATA);
        assert!(res.is_ok());
    }

    #[test]
    fn nonexistant_func() {
        let res = process("{doesntexist}", &cb, &EMPTY_DATA);
        assert!(!res.is_ok());
    }

    #[test]
    fn escaped_func() {
        let res = process("\\{dontcall}", &cb, &EMPTY_DATA);
        assert!(res.is_ok());
        if let Ok(str) = res {
            assert_eq!(str, "{dontcall"); // missing } is intentional; sharex parser also messes
                                          // this up
        }
    }
}
