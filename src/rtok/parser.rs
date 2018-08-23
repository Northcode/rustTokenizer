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

pub struct Parser {
    input: Vec<TokenValue>,
    pstack: Vec<ParseValue>,
    pub output: Vec<AstNode>,
    // reductions: Vec<(Box<Fn(&Vec<ParseValue>) -> bool>, Box<Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError>>)>
}

enum ParseAction {
    Reduce(usize),
    Shift,
    Stop,
}

pub enum ParseError {
    EOF, NoActions
}

impl Parser {

    pub fn new(input: Vec<TokenValue>) -> Parser {
        Parser { input, pstack: Vec::new(), output: Vec::new() }
    }

    pub fn debug_print_stack(&self) {
        println!("stack: {:?}", self.pstack);
    }

    fn shift(&mut self) -> Result<(), ParseError> {
        let val = self.input.pop().ok_or(ParseError::EOF)?;
        self.pstack.push(ParseValue::Token(val));
        Ok(())
    }

    fn reduce_add(vals: &mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
        let _op = vals.pop().ok_or(ParseError::EOF)?;
        let left = vals.pop().ok_or(ParseError::EOF)?;
        let right = vals.pop().ok_or(ParseError::EOF)?;
        Ok(ParseValue::Reduced(AstNode::Add(Box::new(right.into()), Box::new(left.into()))))
    }

    fn determine_reduce_add(vals: &Vec<ParseValue>) -> bool {
        if vals.len() < 3 { return false }

        let lasttwo = &vals[vals.len()-3..vals.len()];
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
        let _last = vals.pop().ok_or(ParseError::EOF)?;
        Ok(ParseValue::Reduced(AstNode::Int))
    }

    fn determine_reduce_int(vals: &Vec<ParseValue>) -> bool {
        vals.last().map_or(false, |i| match i { &ParseValue::Token(TokenValue::Int) => true, _ => false })
    }

    fn reduce<R>(&mut self, reducer: R) -> Result<(), ParseError> where R : Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
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
            ParseAction::Stop => {
                self.output = self.pstack.drain(..).flat_map(|i| match i {
                    ParseValue::Reduced(a) => Some(a),
                    ParseValue::Token(_) => None
                }).collect();
                Err(ParseError::EOF)
            }
            _ => Err(ParseError::NoActions),
        }
    }
}
