
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

#[derive(Debug)]
pub enum ParseError {
    EOF, NoActions, InvalidReduction(usize), InvalidToken, NotImpl
}

impl <T,N> Parser<T,N> {

    pub fn new(input: Vec<T>) -> Parser<T,N> {
        Parser { input, pstack: Vec::new(), output: Vec::new(), reductions: Vec::new() }
    }

    pub fn push_input(&mut self, new_input: Vec<T>) {
        let mut new_vec = new_input;
        // new_vec.append(&mut self.input);
        self.input = new_vec;
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
            .filter(|(_,k)| k.0(&self.pstack))
            .map(|(i,_)| i)
            .nth(0) {
                ParseAction::Reduce(idx)
        }
        else if self.input.len() > 0 {
            ParseAction::Shift
        } else {
            ParseAction::Stop
        }
    }

    pub fn step(&mut self) -> Result<bool, ParseError> {
        let action = self.determine_action();

        match action {
            ParseAction::Shift => self.shift().map(|_| true),
            ParseAction::Reduce(n) => self.reduce(n).map(|_| true),
            ParseAction::Stop => {
                println!("Parser stop");
                self.output = self.pstack.drain(..).flat_map(|i| match i {
                    ParseValue::Reduced(a) => Some(a),
                    ParseValue::Token(_) => None
                }).collect();
                Ok(false)
            }
        }
    }
}

extern crate std;

impl <T,N> std::fmt::Display for ParseValue<T,N> where T: std::fmt::Display, N: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseValue::Token(t) => write!(f, "{}", t),
            ParseValue::Reduced(r) => write!(f, "{}", r),
        }
    }
}

impl <T,N> Parser<T,N> where T: std::fmt::Display, N : std::fmt::Display {
    pub fn print_stack(&self) {
        let formatted = self.pstack.iter().map(|i| format!("{}", i)).collect::<Vec<String>>().join(",");
        println!("stack: [{}]", formatted);
        // println!("stack: {}", self.pstack);
    }

}

impl <T,N> Parser<T,N> where T: std::fmt::Debug, N : std::fmt::Debug {
    pub fn debug_print_stack(&self) {
        // println!("input: {:?}, stack: {:?}, out: {:?}", self.input, self.pstack, self.output);
        println!("stack: {:?}",  self.pstack);
    }

}


#[macro_export]
macro_rules! expect_inner {
    (t $tok:pat) => {&ParseValue::Token($tok)};
    (n $tok:pat) => {&ParseValue::Reduced($tok)};
}

#[macro_export]
macro_rules! expect {
    ($($($type:ident $thing:pat)|+),*) => {|stack| {
        let mut itr = stack.iter().rev();

        let mut is_match = true;

        $({
            {
                is_match = is_match && (match itr.next() {
                    $(
                        Some(expect_inner!($type $thing)) => true,
                    )+
                    _ => false
                })
            }
        })+

            return is_match
    }};
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! reduction_inner {
    ($inner:expr) => {
        return Ok(ParseValue::Reduced($inner))
    };
    ($stack:ident $inner:expr; $pat:pat) => {
        if let $pat = $stack.pop().ok_or(ParseError::EOF)? {
            reduction_inner!($inner);
        }
    };
    ($stack:ident $inner:expr; $pat_first:pat, $($pat_tail:pat),*) => {
        if let $pat_first = $stack.pop().ok_or(ParseError::EOF)? {
            reduction_inner!($stack $inner; $($pat_tail),*)
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! reduction {
    ($inner:expr; $pat:pat) => {
        |stack| {
            reduction_inner!(stack $inner; $pat);

            return Err(ParseError::InvalidToken);
        }
    };
    ($inner:expr; $pat_first:pat, $($pat_tail:pat),*) => {
        |stack| {
            reduction_inner!(stack $inner; $pat_first, $($pat_tail),*);

            return Err(ParseError::InvalidToken);
        }
    };
}

#[macro_export]
macro_rules! wrap_intos {
    ($parser:ident; $($wrapper:pat),*) => {
        $(
            $parser.add_rule(
                expect!(t $wrapper),
                reduction!(i.into(); ParseValue::Token(i)));
        )*
    }
}
