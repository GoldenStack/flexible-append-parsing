pub mod context;

use std::fmt::Display;

use context::Context;

fn main() {
    let root = parse(&mut "@a,#b:c.q").unwrap();

    println!("{root}");

    println!("(@ ((, a) ((. (# ((: b) c))) q)))");
    println!("{}", "(@ ((, a) ((. (# ((: b) c))) q)))" == format!("{root}"))
}

/// Returns the value of the leftmost node in an expression. This is possible
/// because the leftmost node must be an [Expr::Name].
pub fn leftmost(expr: &Expr) -> &String {
    match expr {
        Expr::Name(name) => name,
        Expr::App(left, _) => leftmost(left)
    }
}

pub fn parse(input: &mut &str) -> Result<Box<Expr>> {
    let mut base = expr_token(input)?;

    loop {
        let Ok(next) = expr_token(input) else {
            return Ok(base);
        };

        base = append(base, next);
    }
}

fn associativity(a: &Expr, b: &Expr) -> Associativity {
    Context::standard().get_associativity(&leftmost(a), &leftmost(b)).unwrap()
}

fn infix(e: &Expr) -> bool {
    matches!(e, Expr::App(l, _) if matches!(l.as_ref(), Expr::Name(n) if Context::standard().is_infix(&n)))
}

fn combine(l: Box<Expr>, r: Box<Expr>) -> Box<Expr> {
    let flip = matches!(r.as_ref(), Expr::Name(t) if Context::standard().is_infix(&t));

    Box::new(if flip {
        Expr::App(r, l)
    } else {
        Expr::App(l, r)
    })
}

fn append(base: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    match associativity(&base, &right) {
        Associativity::Left => {
            let res = append_left(base, right);
            if let Some(expr) = res.1 {
                combine(res.0, expr)
            } else {
                res.0
            }
        },
        Associativity::Right => append_right(base, right)
    }
}

fn append_left(mut base: Box<Expr>, right: Box<Expr>) -> (Box<Expr>, Option<Box<Expr>>) {
    if infix(&base) {
        return (combine(base, right), None);
    }

    match associativity(&base, &right) {
        Associativity::Left => {
            match *base {
                Expr::Name(_) => (combine(base, right), None),
                Expr::App(l, r) => {
                    let res = append_left(r, right);
                    *base = Expr::App(l, res.0);
                    (base, res.1)
                },
            }
        }
        Associativity::Right => (base, Some(right)),
    }
}

fn append_right(mut base: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    if infix(&base) {
        return combine(base, right);
    }

    match associativity(&base, &right) {
        Associativity::Left => combine(base, right),
        Associativity::Right => {
            match *base {
                Expr::Name(_) => combine(base, right),
                Expr::App(l, r) => {
                    let res = append_right(r, right);
                    *base = Expr::App(l, res);
                    base
                }
            }
        }
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

pub fn expr_token(src: &mut &str) -> Result<Box<Expr>> {
    token(src).map(Expr::Name).map(Box::new)
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

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Associativity {
    Left,
    Right,
}