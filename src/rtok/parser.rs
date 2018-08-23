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

    reductions: Vec<(Box<Fn(&Vec<ParseValue>) -> bool>, 
                     Box<Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError>>)>
}

enum ParseAction {
    Reduce(usize),
    Shift,
    Stop,
}

pub enum ParseError {
    EOF, NoActions, InvalidReduction(usize),
}

impl Parser {

    pub fn new(input: Vec<TokenValue>) -> Parser {
        Parser { input, pstack: Vec::new(), output: Vec::new(), reductions: Vec::new() }
    }

    pub fn debug_print_stack(&self) {
        println!("stack: {:?}", self.pstack);
    }

    fn shift(&mut self) -> Result<(), ParseError> {
        let val = self.input.pop().ok_or(ParseError::EOF)?;
        self.pstack.push(ParseValue::Token(val));
        Ok(())
    }

    pub fn add_rule<C,R>(&mut self, checker: C, reduction: R)
    where C : 'static + Fn(&Vec<ParseValue>) -> bool , R: 'static + Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
        self.reductions.push((Box::new(checker), Box::new(reduction)));
    }

    // fn reduce<R>(&mut self, reducer: R) -> Result<(), ParseError> where R : Fn(&mut Vec<ParseValue>) -> Result<ParseValue, ParseError> {
    //     let res = reducer(&mut self.pstack)?;
    //     self.pstack.push(res);
    //     Ok(())
    // }

    fn reduce(&mut self, idx: usize) -> Result<(), ParseError> {
        let reduction = self.reductions.get(idx).ok_or(ParseError::InvalidReduction(idx))?;
        let res = reduction.1(&mut self.pstack)?;
        self.pstack.push(res);
        Ok(())
    }

    fn determine_action(&self) -> ParseAction {
        if let Some(idx) = self.reductions.iter()
            .enumerate()
            .filter(|(i,k)| k.0(&self.pstack))
            .map(|(i,k)| i)
            .nth(0) {
                ParseAction::Reduce(idx)
        }
        else if self.input.len() > 0 {
            ParseAction::Shift
        } else {
            ParseAction::Stop
        }
    }

    pub fn step(&mut self) -> Result<(), ParseError> {
        let action = self.determine_action();

        match action {
            ParseAction::Shift => self.shift(),
            ParseAction::Reduce(n) => self.reduce(n),
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
