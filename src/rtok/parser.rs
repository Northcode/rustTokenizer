
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
    EOF, NoActions, InvalidReduction(usize), InvalidToken, NotImpl
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

#[macro_export]
macro_rules! expect_inner {
    (t $tok:pat) => {&ParseValue::Token($tok)};
    (n $tok:pat) => {&ParseValue::Reduced($tok)};
}

#[macro_export]
macro_rules! expect {
    ($($type:ident $thing:pat),*) => {|stack| {
        let mut itr = stack.iter().rev();

        let mut is_match = true;

        $({
            {
                is_match = is_match && (match itr.next() {
                    Some(expect_inner!($type $thing)) => true,
                    _ => false
                })
            }
        })+

            return is_match
    }};
}

macro_rules! reduction_inner {
    ($inner:expr) => {
        return Ok(ParseValue::Reduced($inner))
    };
    ($stack:ident $inner:expr; $id:ident $pat:pat) => {
        if let $pat = $stack.pop().ok_or(ParseError::EOF)? {
            reduction_inner!($inner);
        }
    };
    ($stack:ident $inner:expr; $id_first:ident $pat_first:pat, $($id_tail:ident $pat_tail:pat),*) => {
        if let $pat_first = $stack.pop().ok_or(ParseError::EOF)? {
            reduction_inner!($stack $inner; $($id_tail $pat_tail),*)
        }
    };
}

macro_rules! reduction {
    ($inner:expr; $id:ident -> $pat:pat) => {
        |stack| {
            reduction_inner!(stack $inner; $id $pat);

            return Err(ParseError::InvalidToken);
        }
    };
    ($inner:expr; $id_first:ident -> $pat_first:pat, $($id_tail:ident -> $pat_tail:pat),*) => {
        |stack| {
            reduction_inner!(stack $inner; $id_first $pat_first, $($id_tail $pat_tail),*);

            return Err(ParseError::InvalidToken);
        }
    };
}
