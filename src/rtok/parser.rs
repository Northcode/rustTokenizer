use std::collections::HashMap;

#[derive(Debug)]
pub enum TokenValue {
    Int, Float, Op, Empty
}

type AstNodePtr = Box<AstNode>;

#[derive(Debug)]
pub enum AstNode {
    Int, Float, Add(AstNodePtr, AstNodePtr), Empty
}

#[derive(Debug)]
pub enum ParseValue {
    Token(TokenValue),
    Reduced(AstNode),
}

impl Into<AstNode> for ParseValue {

    fn into(self) -> AstNode {
        match self {
            ParseValue::Token(TokenValue::Int) => AstNode::Int,
            ParseValue::Token(TokenValue::Float) => AstNode::Float,
            _ => AstNode::Empty
        }
    }
}

type ParserState = usize;

pub struct Parser {
    input: Vec<TokenValue>,
    pub pstack: Vec<ParseValue>,
    state: ParserState,
    output: Vec<AstNode>,

    // reductions: Vec<(Box<Fn(&Vec<ParseValue>) -> bool>, Box<Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError>>)>
}

pub enum ParseAction {
    Reduce(usize),
    Shift,
    Stop,
    Error,
}

pub enum ParseResult {
    Good,
    Stop
}
    

pub enum ParseError {
    EOF, NotImpl, NoReductions, NoActions
}

impl Parser {

    pub fn new(input: Vec<TokenValue>) -> Parser {
        Parser { input, pstack: Vec::new(), state: 0, output: Vec::new() }
    }

    pub fn shift(&mut self) -> Result<(), ParseError> {
        let val = self.input.pop().ok_or(ParseError::EOF)?;
        self.pstack.push(ParseValue::Token(val));
        Ok(())
    }

    fn reduce_add(vals: &mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
        let op = vals.pop().ok_or(ParseError::EOF)?;
        let left = vals.pop().ok_or(ParseError::EOF)?;
        let right = vals.pop().ok_or(ParseError::EOF)?;
        Ok(ParseValue::Reduced(AstNode::Add(Box::new(right.into()), Box::new(left.into()))))
    }

    fn determine_reduce_add(vals: &Vec<ParseValue>) -> bool {
        println!("vals len: {}", vals.len());
        if vals.len() < 3 { return false }
        let lasttwo = &vals[vals.len()-3..vals.len()];
        println!("last vals: {:?}", lasttwo);
        (match &lasttwo[2] {
            &ParseValue::Token(TokenValue::Op) => true,
            _ => false
        }) && (match &lasttwo[1] {
            &ParseValue::Reduced(AstNode::Int) => true,
            &ParseValue::Reduced(AstNode::Float) => true,
            _ => false
        }) && (match &lasttwo[0] {
            &ParseValue::Reduced(AstNode::Int) => true,
            &ParseValue::Reduced(AstNode::Float) => true,
            _ => false
        })
    }

    fn reduce_int(vals: &mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
        let last = vals.pop().ok_or(ParseError::EOF)?;
        Ok(ParseValue::Reduced(AstNode::Int))
    }

    fn determine_reduce_int(vals: &Vec<ParseValue>) -> bool {
        vals.last().map_or(false, |i| match i { &ParseValue::Token(TokenValue::Int) => true, _ => false })
    }

    pub fn reduce<R>(&mut self, reducer: R) -> Result<(), ParseError> where R : Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
        let res = reducer(&mut self.pstack)?;
        self.pstack.push(res);
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), ParseError> {
        let action = {
            if Parser::determine_reduce_add(&self.pstack) {
                println!("reduce add");
                ParseAction::Reduce(0)
            } else if Parser::determine_reduce_int(&self.pstack) {
                println!("reduce int");
                ParseAction::Reduce(1)
            } else {
                if self.input.len() > 0 {
                    println!("shift");
                    ParseAction::Shift
                } else {
                    ParseAction::Stop
                }
            }  
        };

        match action {
            ParseAction::Shift => self.shift(),
            ParseAction::Reduce(0) => self.reduce(Parser::reduce_add),
            ParseAction::Reduce(1) => self.reduce(Parser::reduce_int),
            ParseAction::Stop => Err(ParseError::EOF),
            _ => Err(ParseError::NoActions),
        }
    }

    pub fn error(&mut self) -> ParseError {
        ParseError::NotImpl
    }

}
