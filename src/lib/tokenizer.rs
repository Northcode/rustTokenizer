extern crate regex;


pub mod postproc {

    use std::collections::HashMap;
    use lib::tokenizer::tokenizer::{Token, TokenTypeId};

    #[derive(Debug)]
    #[derive(PartialEq)]
    pub struct PostprocErr {
        error: String,
        on_type: TokenTypeId,
    }

    impl PostprocErr {
        pub fn new(token_type: TokenTypeId, error: String) -> PostprocErr {
            PostprocErr { on_type: token_type, error }
        }

        pub fn make<T>(token_type: TokenTypeId, error: String) -> Result<T,PostprocErr> {
            Err(PostprocErr::new(token_type, error))
        }
    }

    pub struct BasicPostProcessor<T> {
        postprocfns: HashMap<TokenTypeId, Box<Fn(Token) -> T>>
    }

    pub trait PostProcessor<T> {
        fn run_on(&self, t: Token) -> Result<T, PostprocErr>;
    }

    impl <T> BasicPostProcessor<T> {
        pub fn new() -> BasicPostProcessor<T> {
            BasicPostProcessor { postprocfns: HashMap::new() }
        }
        
        pub fn add_postprocfn_boxed(&mut self, for_id: TokenTypeId, postprocfn: Box<Fn(Token) -> T>) -> () {
            self.postprocfns.insert(for_id, postprocfn);
        }

        pub fn add_postprocfn<F>(&mut self, for_id: TokenTypeId, postprocfn: F) -> ()
        where F : 'static + Fn(Token) -> T {
            self.add_postprocfn_boxed(for_id, Box::new(postprocfn));
        }
    }

    impl <T> PostProcessor<T> for BasicPostProcessor<T> {
        fn run_on(&self, t: Token) -> Result<T, PostprocErr> {
            if let Some(postprocfn) = self.postprocfns.get(&t.typ) {
                Ok(postprocfn(t))
            } else {
                Err(PostprocErr::new(t.typ, "Failed to find prostprocessor for token type".to_string()))
            }
        }
    }

    impl <T> PostProcessor<T> for BasicPostProcessor<Result<T, PostprocErr>> {
        fn run_on(&self, t: Token) -> Result<T, PostprocErr> {
            if let Some(postprocfn) = self.postprocfns.get(&t.typ) {
                postprocfn(t)
            } else {
                Err(PostprocErr::new(t.typ, "Failed to find prostprocessor for token type".to_string()))
            }
        }
    }
}

pub mod tokenizer {
    use regex::{Regex, Captures};

    pub type TokenTypeId = i32;

    #[derive(Debug)]
    pub struct Token<'a> {
        pub typ: TokenTypeId,
        pub parts: Vec<Option<&'a str>>,
    }

    pub struct Matcher {
        pattern: Regex,
        to_type: TokenTypeId,
    }

    impl Matcher {
        pub fn new(pattern: Regex, to_type: TokenTypeId) -> Matcher {
            Matcher { pattern, to_type }
        }
    }

    #[derive(PartialEq)]
    pub enum MatcherPriority {
        First,
        Longest,
        Shortest,
    }

    pub struct Tokenizer {
        matchers: Vec<Matcher>,
        priority: MatcherPriority,
    }

    impl Tokenizer {
        pub fn new(priority: MatcherPriority) -> Tokenizer {
            Tokenizer { matchers: Vec::new(), priority: priority }
        }

        pub fn make(priority: MatcherPriority, matchers: Vec<(&str, TokenTypeId)>) -> Tokenizer {
            let mut tokenizer = Tokenizer::new(priority);
            for (pattern,type_id) in matchers {
                tokenizer.add_matcher(Matcher::new(Regex::new(pattern).unwrap(), type_id));
            }
            tokenizer
        }

        pub fn add_matcher(&mut self, matcher: Matcher) {
            self.matchers.push(matcher);
        }

        pub fn tokenize<'a>(&self, input: &'a String) -> Vec<Token<'a>> {
            let mut current = input.as_str();

            let mut result = Vec::new();
            while current.len() != 0 {

                let mut currmatch : (TokenTypeId, Option<Captures>) = (0, None);

                for matcher in &self.matchers {

                    fn match_length(m: &Captures) -> usize {
                        m.get(0).unwrap().as_str().len()
                    }

                    let match_ = matcher.pattern.captures(current);

                    if ! match_.is_some() {
                        continue;
                    }

                    let hasmatch = currmatch.1.is_some();

                    if ! hasmatch {
                        currmatch = (matcher.to_type, match_);
                        if self.priority == MatcherPriority::First {
                            break;
                        } else {
                            continue;
                        }
                    }

                    let currentlen = match_length(currmatch.1.as_ref().unwrap());
                    let nextlen = match_length(match_.as_ref().unwrap());

                    match self.priority {
                        MatcherPriority::Longest => {
                            if currentlen < nextlen {
                                currmatch = (matcher.to_type, match_);
                            }
                        },
                        MatcherPriority::First => {},
                        MatcherPriority::Shortest => {
                            if currentlen > nextlen {
                                currmatch = (matcher.to_type, match_);
                            }
                        }
                    }
                }

                if let (mtype, Some(m)) = currmatch {
                    current = &current[m.get(0).unwrap().end()..];
                    result.push( Token { typ: mtype , parts: m.iter().map(|i| i.map(|s| s.as_str())).collect() });
                } else {
                    break;
                }
            }

            return result;
        }
    }
}
