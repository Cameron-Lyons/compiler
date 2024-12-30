#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Null,
}
