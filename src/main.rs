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
    Int(i32), Float(f32), Add(TestAstPtr, TestAstPtr), Sub(TestAstPtr, TestAstPtr), Empty
}

impl std::fmt::Display for TestAst {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TestAst::*;
        match self {
            Int(i) => write!(f, "{}", i),
            Float(i) => write!(f, "{}", i),
            Add(a,b) => write!(f, "(+ {} {})", a, b),
            Sub(a,b) => write!(f, "(- {} {})", a, b),
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

        // rule for add
        parser.add_rule(
            expect!(t T_::Op('+'),
                    n N_::Int(_) | n N_::Float(_) | n N_::Add(..) | n N_::Sub(..),
                    n N_::Int(_) | n N_::Float(_) | n N_::Add(..) | n N_::Sub(..)),
            reduction!(N_::Add(Box::new(left), Box::new(right)); 
                       _o -> PT_(T_::Op(_)), 
                       left -> PR_(left),
                       right -> PR_(right)));

        // rule for sub
        parser.add_rule(
            expect!(t T_::Op('-'),
                    n N_::Int(_) | n N_::Float(_) | n N_::Add(..) | n N_::Sub(..),
                    n N_::Int(_) | n N_::Float(_) | n N_::Add(..) | n N_::Sub(..)),
            reduction!(N_::Sub(Box::new(left), Box::new(right)); 
                       _o -> PT_(T_::Op(_)), 
                       left -> PR_(left),
                       right -> PR_(right)));
    }

    let mut line = String::new();
    while let Ok(n) = io::stdin().read_line(&mut line) {
        let tokens = tokenize_str(&line);
        line.clear();

        parser.push_input(tokens.into_iter().rev().collect());

        while parser.step().is_ok() {
            // parser.debug_print_stack();
            parser.print_stack();
        }


        println!("{:?}", parser.output);
    }


}
