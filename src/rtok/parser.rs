
#[derive(Debug)]
pub enum ParseValue<T,N> {
    Token(T),
    Reduced(N),
}

pub struct Parser<T,N> {

    input: Vec<T>,
    pstack: Vec<ParseValue<T,N>>,
    pub output: Vec<N>,

    reductions: Vec<(Box<Fn(&Vec<ParseValue<T,N>>) -> bool>, 
                     Box<Fn(&mut Vec<ParseValue<T,N>>) -> Result<ParseValue<T,N>, ParseError>>)>
}

enum ParseAction {
    Reduce(usize),
    Shift,
    Stop,
}

pub enum ParseError {
    EOF, NoActions, InvalidReduction(usize), InvalidToken
}

impl <T,N> Parser<T,N> {

    pub fn new(input: Vec<T>) -> Parser<T,N> {
        Parser { input, pstack: Vec::new(), output: Vec::new(), reductions: Vec::new() }
    }

    fn shift(&mut self) -> Result<(), ParseError> {
        let val = self.input.pop().ok_or(ParseError::EOF)?;
        self.pstack.push(ParseValue::Token(val));
        Ok(())
    }

    pub fn add_rule<C,R>(&mut self, checker: C, reduction: R)
    where C : 'static + Fn(&Vec<ParseValue<T,N>>) -> bool , R: 'static + Fn(&mut Vec<ParseValue<T,N>>) -> Result<ParseValue<T,N>, ParseError> {
        self.reductions.push((Box::new(checker), Box::new(reduction)));
    }

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

extern crate std;

impl <T,N> Parser<T,N> where T: std::fmt::Debug, N : std::fmt::Debug {
    pub fn debug_print_stack(&self) {
        println!("stack: {:?}", self.pstack);
    }

}
