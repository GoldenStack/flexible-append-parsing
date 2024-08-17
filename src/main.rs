
fn main() {
    println!("Hello, world!");

    let src = &mut "  Hello, world!";

    println!("{:?}", whitespace(src));
    println!("{}", src);
    println!("{:?}", token(src));
}


pub type Result<T> = std::result::Result<T, (String, Error)>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EOF
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
    let copy = &mut src.clone();
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

pub enum Expr {
    Infix(String, Box<Expr>, Box<Expr>),
    Prefix(String, Box<Expr>),
    Literal(String),
}

impl Expr {
    pub fn infix(op: String, left: Expr, right: Expr) -> Self {
        Expr::Infix(op, Box::new(left), Box::new(right))
    }

    pub fn prefix(op: String, right: Expr) -> Self {
        Expr::Prefix(op, Box::new(right))
    }
}