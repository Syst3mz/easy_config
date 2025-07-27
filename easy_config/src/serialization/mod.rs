pub mod primitives;
pub mod tuples;
pub mod serialization_error;
pub mod collections;
pub mod serialize_enum;
pub mod option_span_combine;
pub mod option;
mod enum_helpers;

use std::path::Path;
use crate::expression::Expression;
use crate::expression_iterator::ExpressionIterator;
use crate::parser::Parser;
use crate::serialization::serialization_error::{Kind, SerializationError};

pub trait Config: 'static {
    /// PASSTHROUGH is true for types which need access to their parent's iterators for
    /// deserialization. This is mostly enums, but is exposed for anything else.
    const PASSTHROUGH: bool = false;
    fn serialize(&self) -> Expression;
    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> where Self: Sized;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadMode {
    Loaded,
    Default
}

/// Any structure implementing both `Default` and `Config` automatically implements this trait.
pub trait DefaultConfig: Config + Default {
    /// This function always returns a config (if successful).
    /// If a configuration file is both present (at the specified path) and valid
    /// (can be interpreted without error), then that configuration is returned. If the config is
    /// not present, then a new file is generated at `path` and the default config is stored in it.
    /// If a config is invalid this function will return an error.
    fn deserialize_from_file_or_default_and_write(path: impl AsRef<Path>) -> Result<(Self, LoadMode), SerializationError> where Self: Sized {
        let path = path.as_ref();
        if std::fs::exists(path)? {
            let text = std::fs::read_to_string(path)?;
            let finished_parser = Parser::new(text.as_str()).parse();

            if !finished_parser.errors().is_empty() {
                return Err(SerializationError::FirstLevelError(Kind::ParserErrors(finished_parser.errors().clone()), String::new()));
            }

            let expr  = finished_parser.unwrap();

            Ok((Self::deserialize(&mut expr.into_iter(), text)?, LoadMode::Loaded))
        } else {
            let default = Self::default();

            std::fs::write(path, default.serialize().pretty())?;

            Ok((default, LoadMode::Default))
        }
    }
}

impl<T: Default + Config> DefaultConfig for T {}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use crate::serialization::enum_helpers::get_discriminant_lowercased;
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Address {
        None,
        IpV4(String),
        Index(u32)
    }

    impl Address {
        fn deserialize_fields(discriminant: &str, iter: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
            let source_text = source_text.as_ref();
            match discriminant {
                "none" => Ok(Address::None),
                "ipv4" => Ok(Address::IpV4(String::deserialize(iter, source_text)?)),
                "index" => Ok(Address::Index(u32::deserialize(iter, source_text)?)),
                _ => unreachable!()
            }
        }
    }
    impl Config for Address {
        const PASSTHROUGH: bool = true;
        fn serialize(&self) -> Expression {
            match self {
                Address::None => Expression::presence("None"),
                Address::IpV4(s) => Expression::list(vec![Expression::presence("IpV4"), Expression::list(vec![Expression::presence(s.clone())])]),
                Address::Index(u) => Expression::list(vec![Expression::presence("Index"), Expression::list(vec![Expression::presence(*u)])])
            }
        }

        fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
        where
            Self: Sized
        {
            let source_text = source_text.as_ref();
            let options = &["none", "ipv4", "index"];
            let discriminant = get_discriminant_lowercased(expression_iterator, options, source_text)?;

            // hack to allow prefixed exprs
            if let Some(expr) = expression_iterator.peek() {
                if expr.is_list() {
                    let mut iter = expression_iterator.next().unwrap().into_iter();
                    let ret = Self::deserialize_fields(discriminant.as_str(), &mut iter, source_text);
                    expression_iterator.rewind(dbg!(iter.len()));
                    return ret
                }
            }

            Self::deserialize_fields(discriminant.as_str(), expression_iterator, source_text)
        }
    }
    
    #[derive(Debug, PartialEq)]
    struct Demo {
        name: String,
        addresses: Vec<Address>,
    }

    impl Config for Demo {
        fn serialize(&self) -> Expression {
            Expression::list(vec![
                Expression::binding("name", self.name.serialize()),
                Expression::binding("addresses", self.addresses.serialize())
            ])
        }

        fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
        where
            Self: Sized,
        {
            let source_text = source_text.as_ref();

            Ok(Self {
                name: String::deserialize(&mut exprs.find_binding("name", source_text)?.value.into_iter(), source_text)?,
                addresses: Vec::<Address>::deserialize(&mut exprs.find_binding("addresses", source_text)?.value.into_iter(), source_text)?,
            })
        }
    }


    fn demo() -> Demo {
        Demo {
            name: "cat".to_string(),
            addresses: vec![
                Address::None,
                Address::IpV4("127.0.0.1".to_string()),
                Address::Index(3),
            ],
        }
    }

    const EXPECTED: &'static str = "(name = cat addresses = (None (IpV4 (127.0.0.1)) (Index (3))))";
    #[test]
    fn serialize() {
        let d = demo();
        assert_eq!(d.serialize().minimized().dump(), EXPECTED)
    }

    #[test]
    fn deserialize() {
        let parsed = Parser::new(EXPECTED).parse().unwrap();
        assert_eq!(
            Demo::deserialize(&mut parsed.into_iter().next_or_err(EXPECTED).unwrap().into_iter(), EXPECTED).unwrap(), demo()
        )
    }

    #[should_panic]
    #[test]
    fn deserialize_err() {
        let parsed = Parser::new("()").parse().unwrap();
        assert_eq!(
            Demo::deserialize(&mut parsed.into_iter().next_or_err(EXPECTED).unwrap().into_iter(), "()").unwrap(), demo()
        )
    }
}