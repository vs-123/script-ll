use std::process;

#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    None,
    LexingError(String),
    RuntimeError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn print_error(msg: String) {
    println!("{}", msg);
    process::exit(1);
}
