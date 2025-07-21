mod lexer;
pub mod expression;
#[allow(dead_code)]
pub mod parser;
pub mod serialization;
pub mod lexical_range;

#[cfg(feature = "derive")]
pub use easy_config_derive::Config;