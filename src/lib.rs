extern crate regex;

#[cfg(test)]
mod lib {

    mod tokenizer;
    use self::tokenizer::tokenizer::{Tokenizer, MatcherPriority, Token};
    use self::tokenizer::postproc::{BasicPostProcessor, PostProcessor, PostprocErr};

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
        enum TokenValue {
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
            Ok(TokenValue::Whitespace)
        });

        postproc.add_postprocfn(1, |t| {
            let tokenstr = get_token_part(&t, 1)?;
            tokenstr.parse()
                .map(|i| TokenValue::Int(i))
                .or(PostprocErr::make(t.typ, "Failed to parse token as int".to_string()))
        });

        postproc.add_postprocfn(2, |t| {
            let tokenstr = get_token_part(&t, 1)?;

            tokenstr.parse()
                .map(|i| TokenValue::Float(i))
                .or(PostprocErr::make(t.typ, "Failed to parse token as float".to_string()))
        });




        fn not_whitespace(t: &Result<TokenValue,PostprocErr>) -> bool {
            match t {
                Ok(TokenValue::Whitespace) => false,
                _ => true
            }
        }

        let tokenvals : Vec<Result<TokenValue, PostprocErr>> = tokens.into_iter()
            .map(|i| postproc.run_on(i))
            .filter(not_whitespace)
            .collect();



        println!("tokenvals: {:?}", tokenvals);

        assert_eq!(tokenvals.get(0), Some(&Ok(TokenValue::Int(1))));
        assert_eq!(tokenvals.get(2), Some(&Ok(TokenValue::Float(3.14))));
    }
}
