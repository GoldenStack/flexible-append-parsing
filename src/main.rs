use context::Context;

pub mod context;


fn main() {
    let src = &mut "-a.b";

    println!("{:?}", parse(src));
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

        let order = Context::standard().get_associativity(leftmost(&left), &right, input)?;

        // If it's right associative relative to the left element we pair it
        // up with the element after it. Otherwise we ignore the element
        // after and allow following loop iterations to handle it.
        left = Expr::App(Box::new(left), Box::new(match order {
                Associativity::Right => parse_prefix(Expr::Name(right), input)?,
                Associativity::Left => Expr::Name(right)
        }));
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

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Associativity {
    Left,
    Right,
}