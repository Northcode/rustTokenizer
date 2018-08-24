extern crate regex;

#[macro_use]
mod rtok;

use rtok::tokenizer::{Tokenizer, MatcherPriority, Token};
use rtok::tokenizer::postproc::{BasicPostProcessor, PostProcessor, PostprocErr};

#[derive(Debug)]
#[derive(PartialEq)]
enum TestTokenValue {
    Int(i32),
    Float(f32),
    Op(char),
    Whitespace
}

impl std::fmt::Display for TestTokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TestTokenValue::*;
        match self {
            Int(i) => write!(f, "{}", i),
            Float(i) => write!(f, "{}", i),
            Op(i) => write!(f, "{}", i),
            Whitespace => write!(f, " "),
        }
    }
}

fn tokenize_str(s: &str) -> Vec<TestTokenValue> {
    let tokenizer = Tokenizer::make(MatcherPriority::Longest, vec![(r"^(\s+)", 0), (r"^(\d+)", 1), (r"^(\d+\.\d+)",2), (r"([+\-*/])", 3)]);

    let startstr = String::from(s);

    let tokens = tokenizer.tokenize(&startstr);


    let mut postproc = BasicPostProcessor::new();


    fn get_token_part<'a>(t: &'a Token, i: usize) -> Result<&'a str, PostprocErr> {
        if let Some(Some(part)) = t.parts.get(i) {
            Ok(part)
        } else {
            PostprocErr::make(t.typ, format!("Failed to get token part: {}", i))
        }
    }

    postproc.add_postprocfn(0, |_| {
        Ok(TestTokenValue::Whitespace)
    });

    postproc.add_postprocfn(1, |t| {
        let tokenstr = get_token_part(&t, 1)?;
        tokenstr.parse()
            .map(|i| TestTokenValue::Int(i))
            .or(PostprocErr::make(t.typ, "Failed to parse token as int".to_string()))
    });

    postproc.add_postprocfn(2, |t| {
        let tokenstr = get_token_part(&t, 1)?;

        tokenstr.parse()
            .map(|i| TestTokenValue::Float(i))
            .or(PostprocErr::make(t.typ, "Failed to parse token as float".to_string()))
    });
    postproc.add_postprocfn(3, |t| {
        let tokenstr = get_token_part(&t, 1)?;

        tokenstr.chars().nth(0)
            .map(|i| TestTokenValue::Op(i))
            .ok_or(PostprocErr::new(t.typ, "Failed to get operator from token".to_string()))
    });


    fn not_whitespace(t: &Result<TestTokenValue,PostprocErr>) -> bool {
        match t {
            Ok(TestTokenValue::Whitespace) => false,
            _ => true
        }
    }

    let tokenvals : Vec<TestTokenValue> = tokens.into_iter()
        .map(|i| postproc.run_on(i))
        .flat_map(|i| { i.into_iter() })
        .filter(|i| i != &TestTokenValue::Whitespace)
        .collect();

    tokenvals
}

use rtok::parser::{Parser};

type TestAstPtr = Box<TestAst>;

#[derive(Debug)]
pub enum TestAst {
    Int(i32), Float(f32), Value(TestAstPtr), Add(TestAstPtr, TestAstPtr), Sub(TestAstPtr, TestAstPtr), Empty
}

impl std::fmt::Display for TestAst {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TestAst::*;
        match self {
            Int(i) => write!(f, "{}", i),
            Float(i) => write!(f, "{}", i),
            Add(a,b) => write!(f, "(+ {} {})", a, b),
            Sub(a,b) => write!(f, "(- {} {})", a, b),
            Value(v) => write!(f, "{}", v),
            Empty => write!(f, "EMPTY"),
        }
    }
}

impl Into<TestAst> for TestTokenValue {
    fn into(self) -> TestAst {
        match self {
            TestTokenValue::Int(i) => TestAst::Int(i),
            TestTokenValue::Float(i) => TestAst::Float(i),
            _ => TestAst::Empty
        }
    }
}

use std::io;

fn main() {
    let tokenvals = tokenize_str("");

    // assert_eq!(tokenvals.get(2), Some(&TestTokenValue::Op('+')));

    let mut parser : Parser<TestTokenValue, TestAst> = Parser::new(tokenvals.into_iter().rev().collect());

    use self::TestTokenValue as T_;
    use self::TestAst as N_;
    use rtok::parser::ParseValue::Token as PT_;
    use rtok::parser::ParseValue::Reduced as PR_;

    {
        use rtok::parser::{ParseError, ParseValue};

        use TestTokenValue as T_;
        use TestAst as N_;

        // auto add rules for tokens that impl Into<TestAst>
        // needs patterns that match the tokens
        wrap_intos!(parser; T_::Float(_), T_::Int(_));

        parser.add_rule(
            expect!(n N_::Int(_) | n N_::Float(_) | n N_::Add(..) | n N_::Sub(..)),
            reduction!(N_::Value(Box::new(inner));
                       inner -> PR_(inner)));

        // rule for add
        parser.add_rule(
            expect!(n N_::Value(_),
                    t T_::Op('+'),
                    n N_::Value(_)),
            reduction!(N_::Add(Box::new(left), Box::new(right)); 
                       left -> PR_(left),
                       _o -> PT_(T_::Op(_)), 
                       right -> PR_(right)));

        // rule for sub
        parser.add_rule(
            expect!(n N_::Value(_),
                    t T_::Op('-'),
                    n N_::Value(_)),
            reduction!(N_::Sub(Box::new(left), Box::new(right)); 
                       left -> PR_(left),
                       _o -> PT_(T_::Op(_)), 
                       right -> PR_(right)));
    }

    let mut line = String::new();
    while let Ok(n) = io::stdin().read_line(&mut line) {
        let tokens = tokenize_str(&line);
        line.clear();

        parser.push_input(tokens.into_iter().rev().collect());

        while parser.step().is_ok() {
            // parser.debug_print_stack();
            // parser.print_stack();
        }

        let out : Vec<N_> = parser.output.drain(..).collect();


        fn code_gen_node(buff: &mut String, node: N_) {
            match node {
                N_::Add(a,b) => {
                    code_gen_node(buff, *b);
                    write!(buff, "+");
                    code_gen_node(buff, *a);
                },
                N_::Sub(a,b) => {
                    code_gen_node(buff, *b);
                    write!(buff, "-");
                    code_gen_node(buff, *a);
                },
                N_::Value(v) => {
                    code_gen_node(buff, *v);
                },
                N_::Int(a) => {
                    write!(buff, "{}", a);
                },
                N_::Float(a) => {
                    write!(buff, "{}", a);
                },
                N_::Empty => {
                    write!(buff, "");
                },
            }
        }

        let mut output = String::new();
        use std::fmt::Write;
        write!(&mut output, "#include \"stdio.h\" \n\n");
        write!(&mut output, "{}", "int main() {\n");
        write!(&mut output, "{}", "  int res = ");

        for node in out {
            code_gen_node(&mut output, node);
        }

        write!(&mut output, "{}", "; \n printf(\"%d\", res); return 0; }");

        fn write_code_to_file(s : &String) -> std::io::Result<()> {
            use std::fs::File;
            use std::io::Write;

            let mut file = File::create("test.c")?;
            file.write_all(s.as_bytes())?;

            Ok(())
        }

        write_code_to_file(&output).expect("Failed to write code to file");

        use std::process::Command;

        let cmdout = Command::new("gcc")
            .arg("-o").arg("_test")
            .arg("test.c")
            .output()
            .expect("Gcc invokation failed, do you have it installed?");

        let prgout = Command::new("./_test")
            .output();

        if let Err(e) = prgout {
            println!("Program invokation failed, maybe it failed to build, displaying gcc output now...");
            println!("GCC: {:?}", cmdout);
        } else if let Ok(prgout) = prgout {
            println!("Result: {}", String::from_utf8_lossy(&prgout.stdout));
        }

    }


}
