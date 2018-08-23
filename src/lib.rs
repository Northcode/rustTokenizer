extern crate regex;

mod rtok;

#[cfg(test)]
mod lib {

    use rtok::tokenizer::{Tokenizer, MatcherPriority, Token};
    use rtok::tokenizer::postproc::{BasicPostProcessor, PostProcessor, PostprocErr};

    #[test]
    fn test_whitespace() {
        let tokenizer = Tokenizer::make(MatcherPriority::First, vec![(r"(\s+)", 0)]);
        // tokenizer.add_matcher(Matcher::new(Regex::new(r"(\s+)").unwrap(), 0));

        let startstr = String::from("this is a test");

        let tokens = tokenizer.tokenize(&startstr);

        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_numbers() {
        let tokenizer = Tokenizer::make(MatcherPriority::First, vec![(r"^(\s+)", 0), (r"^(\d+\.\d+)",2), (r"^(\d+)", 1)]);

        let startstr = String::from("1 2 3.14 5.123 123");

        let tokens = tokenizer.tokenize(&startstr);

        assert_eq!(tokens.len(),9);
    }

    #[test]
    fn test_priority_longest() {
        let tokenizer = Tokenizer::make(MatcherPriority::Longest, vec![(r"^(\s+)", 0), (r"^(\d+)", 1), (r"^(\d+\.\d+)",2)]);

        let startstr = String::from("1 234245 3.14 5.123 123");

        let tokens = tokenizer.tokenize(&startstr);

        assert_eq!(tokens.len(), 9);
    }

    #[test]
    fn test_priority_shortest() {
        let tokenizer = Tokenizer::make(MatcherPriority::Shortest, vec![(r"^(\s+)", 0), (r"^(\d+\.\d+)",2), (r"^(\d+)", 1)]);

        let startstr = String::from("1 234245 3.14 5.123 123");

        let tokens = tokenizer.tokenize(&startstr);

        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_conversion() {
        let tokenizer = Tokenizer::make(MatcherPriority::Longest, vec![(r"^(\s+)", 0), (r"^(\d+)", 1), (r"^(\d+\.\d+)",2)]);

        let startstr = String::from("1 234245 3.14 5.123 123");

        let tokens = tokenizer.tokenize(&startstr);

        #[derive(Debug)]
        #[derive(PartialEq)]
        enum TestTokenValue {
            Int(i32),
            Float(f32),
            Whitespace
        };


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




        fn not_whitespace(t: &Result<TestTokenValue,PostprocErr>) -> bool {
            match t {
                Ok(TestTokenValue::Whitespace) => false,
                _ => true
            }
        }

        let tokenvals : Vec<Result<TestTokenValue, PostprocErr>> = tokens.into_iter()
            .map(|i| postproc.run_on(i))
            .filter(not_whitespace)
            .collect();



        println!("tokenvals: {:?}", tokenvals);

        assert_eq!(tokenvals.get(0), Some(&Ok(TestTokenValue::Int(1))));
        assert_eq!(tokenvals.get(2), Some(&Ok(TestTokenValue::Float(3.14))));
    }


    mod parsetest {

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

        use rtok::parser::{Parser,AstNode};
        use rtok::parser::TokenValue;

        #[test]
        fn test_parser() {
            let tokenvals = tokenize_str("1 2 +");

            assert_eq!(tokenvals.get(2), Some(&TestTokenValue::Op('+')));

            let mut parser = Parser::new(tokenvals.into_iter().rev().map(|i| match i {
                TestTokenValue::Int(_) => TokenValue::Int,
                TestTokenValue::Float(_) => TokenValue::Float,
                TestTokenValue::Op(_) => TokenValue::Op,
                _ => TokenValue::Empty
            }).collect());

            while parser.step().is_ok() {
                parser.debug_print_stack();
            }

            match parser.output.last() {
                Some(&AstNode::Add(..)) => {
                    assert!(true)
                }
                _ => assert!(false)
            }
        }
    }
}
