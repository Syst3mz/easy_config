mod lexer;
mod str_extensions;

pub enum Expression {
    Presence(String),
    Pair(String, Box<Expression>),
    Collection(Vec<Expression>)
}