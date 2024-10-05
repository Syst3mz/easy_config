use crate::parser::Parser;
use crate::serialization::Config;
use crate::serialization::error::Error;

#[allow(dead_code)]
pub trait StringExtension<T> {
    fn deserialize(&self) -> Result<T, Error>;
}

#[allow(dead_code)]
impl<T: AsRef<str>, R: Config> StringExtension<R> for T {
    fn deserialize(&self) -> Result<R, Error> {
        R::deserialize(Parser::new(self).parse()?)
    }
}