use std::fmt::Display;

use context::Context;

pub mod context;


fn main() {
    let src = &mut "@a q.b";

    // @a q.b is (@(a q)).b
    // -a q.b is -((a q).b)
    
    // desired:
    // (@(a q)).b

    // gt(a, @)
    // gt(@, .)


    // let src = &mut "@a q.b";

    // (- ((. (a q)) b))
    // (@ ((. (a q)) b))

    println!("{}", parse(src).unwrap());
}

/// Returns the value of the leftmost node in an expression. This is possible
/// because the leftmost node must be an [Expr::Name].
pub fn leftmost(expr: &Expr) -> &String {
    match expr {
        Expr::Name(name) => name,
        Expr::App(left, _) => leftmost(left)
    }
}

pub fn parse(input: &mut &str) -> Result<Expr> {
    parse_prefix(token(input).map(Expr::Name)?, input)
}

/// Parses prefix expressions (e.g. `a b c`).
pub fn parse_prefix(mut left: Expr, input: &mut &str) -> Result<Expr> {
    loop {
        // If there isn't another token, just exit
        let Ok(right) = token(input) else {
            return Ok(left);
        };

        
        println!("[1] left: {left}; right: {right}; prec: {:?}", Context::standard().get_associativity(leftmost(&left), &right, input)?);
        if Context::standard().is_infix(&right) {
            // at this point it is necessary to steal from the left-hand side
            println!("[2] branching at {} and {}:", leftmost(&left), &right);
            match Context::standard().get_associativity(leftmost(&left), &right, input)? {
                Associativity::Left => {
                    println!("[3] left after left: {right} {left}");
                    left = parse_prefix(Expr::Name(right).app(left), input)?;
                    continue;
                },
                Associativity::Right => {
                    println!("[4] after right: left {left}; right {right}");
                    left = match left {
                        Expr::Name(_) => todo!(),
                        Expr::App(l, r) => l.app(parse_prefix(Expr::Name(right).app(*r), input)?)
                    };
                    continue;
                }
            }
        };

        let order = Context::standard().get_associativity(leftmost(&left), &right, input)?;


        if order == Associativity::Left {
            println!("[5] {left} applied to {right}; looping because {left} > {right}");
        } else {
            println!("[6] parsing with left as {right}, input as /{input}/ because {left} < {right}");
        }
        // If it's right associative relative to the left element we pair it
        // up with the element after it. Otherwise we ignore the element
        // after and allow following loop iterations to handle it.
        left = left.app(match order {
                Associativity::Right => parse_prefix(Expr::Name(right), input)?,
                Associativity::Left => Expr::Name(right)
        });
    }
}

/// Reads the next `char` from the given string, optionally returning the result.
pub fn next(src: &mut &str) -> Option<char> {
    let first = src.chars().next()?;
    *src = &mut &src[first.len_utf8()..];
    Some(first)
}

/// Applies the provided function to a copy of the given string, returning the
/// result.
pub fn peek<T, F: Fn(&mut &str) -> T>(src: &&str, function: F) -> T {
    let copy = &mut &src[..];
    let result = function(copy);
    return result;
}

/// Reads characters from the given source as long as `char::is_whitespace`
/// returns true.
pub fn whitespace(src: &mut &str) -> Result<()> {
    while let Some(_) = peek(src, next).filter(|c| c.is_whitespace()) {
        next(src);
    }

    Ok(())
}

/// Reads whitespace, and then reads either any number of alphabetic charcters,
/// or a single non-alphabetic character.
pub fn token(src: &mut &str) -> Result<String> {
    whitespace(src)?;

    let Some(first) = next(src) else {
        return Err((src.to_owned(), Error::EOF));
    };

    let mut str = String::from(first);

    if !first.is_alphabetic() {
        return Ok(str);
    }

    while let Some(n) = peek(src, next).filter(|c| c.is_alphabetic()) {
        next(src);
        str.push(n);
    }

    return Ok(str);
}

pub type Result<T> = std::result::Result<T, (String, Error)>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UndefinedAssociativity(String, String),
    EOF,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Expr {
    Name(String),
    App(Box<Expr>, Box<Expr>)
}

/// Displays an expression.
/// 
/// `Name` types just return the name, while `App` types return stringified
/// forms of the arguments, surrounded by paretheses.
impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Name(str) => write!(f, "{}", str),
            Expr::App(a, b) => write!(f, "({} {})", a, b),
        }
    }
}

impl Expr {
    pub fn app(self, right: Expr) -> Self {
        Expr::App(Box::new(self), Box::new(right))
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Associativity {
    Left,
    Right,
}