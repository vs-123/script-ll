#[derive(PartialEq, Clone, Debug)]
pub enum Types {
    String,
    Number,
    Identifier,
    Unknown,
}

impl std::fmt::Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
