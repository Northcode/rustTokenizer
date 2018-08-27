extern crate regex;

#[macro_use]
mod rtok;

use rtok::tokenizer::{Tokenizer, MatcherPriority, Token};
use rtok::tokenizer::postproc::{BasicPostProcessor, PostProcessor, PostprocErr};
use rtok::parser::{Parser};

#[derive(Debug)]
pub enum TokenType {
    Ident(String), Literal(String), Assign, LeftPar, RightPar, Star
}

type AstPtr = Box<EBNFAst>;
type AstVec = Vec<AstPtr>;
#[derive(Debug)]
pub enum EBNFAst {
    Ident(String), Literal(String), Definition(AstPtr, AstPtr), Single(AstPtr), Double(AstPtr,AstPtr), OptionalLast(AstPtr, AstPtr), Or(AstPtr,AstPtr), Assign, LeftPar, RightPar, Star
}

impl Into<EBNFAst> for TokenType {
    fn into(self) -> EBNFAst {
        match self {
            TokenType::Ident(s)   => EBNFAst::Ident(s),
            TokenType::Literal(s) => EBNFAst::Literal(s),
            TokenType::Assign     => EBNFAst::Assign,
            TokenType::LeftPar    => EBNFAst::LeftPar,
            TokenType::RightPar   => EBNFAst::RightPar,
            TokenType::Star       => EBNFAst::Star,
        }
    }
}

// use std::fmt::{Formatter, Display};

// impl Display for EBNFAst {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        
//     }
// }

use std::io;

fn main() {
    let input = String::from("expr ::= VARIABLE");

    let tokenizer = Tokenizer::make(MatcherPriority::Longest, vec![(r"^(\s+)", 0), 
                                                                   (r"^([a-zA-Z0-9\-_]+)", 1), 
                                                                   (r"^'([^']+)'", 2),
                                                                   (r"^(::=)", 3),
                                                                   (r"^(\()", 4),
                                                                   (r"^(\))", 5),
                                                                   (r"^(\*)", 6)]);
                                    
    let mut postproc = BasicPostProcessor::new();

    fn get_token_part<'a>(t: &'a Token, i: usize) -> Result<&'a str, PostprocErr> {
        if let Some(Some(part)) = t.parts.get(i) {
            Ok(part)
        } else {
            PostprocErr::make(t.typ, format!("Failed to get token part: {}", i))
        }
    }

    postproc.add_postprocfn(1, |t| {
        let id = get_token_part(&t, 1)?;
        Ok(TokenType::Ident(id.to_string()))
    });

    postproc.add_postprocfn(2, |t| {
        let id = get_token_part(&t, 1)?;
        Ok(TokenType::Literal(id.to_string()))
    });

    postproc.add_postprocfn(3, |_| {Ok(TokenType::Assign)});
    postproc.add_postprocfn(4, |_| {Ok(TokenType::LeftPar)});
    postproc.add_postprocfn(5, |_| {Ok(TokenType::RightPar)});
    postproc.add_postprocfn(6, |_| {Ok(TokenType::Star)});
 
    fn not_whitespace(t: &Token) -> bool {
        t.typ != 0
    }

    let tokens : Vec<TokenType> = tokenizer.tokenize(&input)
        .into_iter()
        .filter(not_whitespace)
        .map(|i| postproc.run_on(i))
        .flat_map(|i| i.into_iter())
        .rev()
        .collect();

    let mut parser : Parser<TokenType, EBNFAst> = Parser::new(tokens);

    {
        use rtok::parser::{ParseError, ParseValue};
        use self::TokenType as T;
        use self::EBNFAst as N;
        use rtok::parser::ParseValue::Token as PT;
        use rtok::parser::ParseValue::Reduced as PR;

        wrap_intos!(parser; T::Ident(_), T::Literal(_), T::Assign, T::LeftPar, T::RightPar, T::Star);

        // rule for Definition
        parser.add_rule(
            expect!(n N::Single(..) | n N::Double(..) | n N::OptionalLast(..) | n N::Or(..),
                    n N::Assign,
                    n N::Single(_)),
            reduction!(N::Definition(Box::new(id), Box::new(expr));
                       PR(expr), PR(N::Assign), PR(id)));

        parser.add_rule(
            expect!(n N::Ident(_) | n N::Literal(_)),
            reduction!(N::Single(Box::new(val));
                       PR(val)));
    }

    loop {
        let res = parser.step();

        parser.debug_print_stack();

        match res {
            Ok(true) => continue,
            Ok(false) => break,
            Err(e) => {
                println!("parse error: {:?}", e);
                break;
            }
        }
    }

    println!("{:?}", parser.output);
}
