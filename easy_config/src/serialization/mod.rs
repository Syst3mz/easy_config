pub mod primitives;
pub mod tuples;
pub mod serialization_error;
pub mod collections;
pub mod option_span_combine;
pub mod option;

use std::collections::HashMap;
use std::path::Path;
use crate::expression::Expression;
use crate::expression_iterator::ExpressionIterator;
use crate::lexical_span::LexicalSpan;
use crate::parser::Parser;
use crate::serialization::serialization_error::{Kind, SerializationError};

pub trait EasyConfig: 'static {
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
pub trait DefaultConfig: EasyConfig + Default {
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

impl<T: Default + EasyConfig> DefaultConfig for T {}


pub fn deserialize_field_from_map_or_error<T: EasyConfig>(field: impl AsRef<str>, mapping: &mut HashMap<String, Expression>, span: LexicalSpan, source_text: impl AsRef<str>) -> Result<T, SerializationError> {
    let field = field.as_ref();
    let source_text = source_text.as_ref();

    let value =  mapping
        .remove(field)
        .ok_or(SerializationError::on_span(Kind::MissingField(field.to_string()), span, source_text))?;

    T::deserialize(
       &mut value.into_iter(),
       source_text
    )

}
#[cfg(test)]
mod tests {
    use crate::config_error::Contextualize;
    use crate::parser::Parser;
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Address {
        None,
        IpV4(String),
        Index(u32, i32)
    }

    impl EasyConfig for Address {
        const PASSTHROUGH: bool = true;
        fn serialize(&self) -> Expression {
            match self {
                Address::None => Expression::presence("None"),
                Address::IpV4(s) => Expression::list(vec![Expression::presence("IpV4"), Expression::list(vec![Expression::presence(s.clone())])]),
                Address::Index(u, i) => Expression::list(vec![Expression::presence("Index"), Expression::list(vec![Expression::presence(*u), Expression::presence(*i)])])
            }
        }

        fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
        where
            Self: Sized
        {
            let source_text = source_text.as_ref();
            let (discriminant, fields) = expression_iterator
                .extract_enum(source_text)
                .contextualize("Unable to deserialize enum Address since we can't extract a discriminant and a argument list")?;
            let span = expression_iterator.span().unwrap();
            const OPTIONS: &'static [&'static str] = &["None", "IpV4", "Index"];
            let mut fields = fields.into_iter();

            match discriminant.as_str() {
                "None" => Ok(Self::None),
                "IpV4" => Ok(Self::IpV4(fields
                    .deserialize_next(source_text)
                    .contextualize("Unable to deserialize IpV4 field")?)
                ),
                "Index" => Ok(Self::Index(
                    fields
                        .deserialize_next(source_text)
                        .contextualize("Unable to deserialize Index 0 field")?,
                    fields
                        .deserialize_next(source_text)
                        .contextualize("Unable to deserialize Index 1 field")?
                )),
                _ => Err(SerializationError::on_span(Kind::ExpectedDiscriminant(discriminant, OPTIONS), span, source_text)),
            }
        }
    }
    
    #[derive(Debug, PartialEq)]
    struct Demo {
        name: String,
        addresses: Vec<Address>,
    }

    impl EasyConfig for Demo {
        fn serialize(&self) -> Expression {
            Expression::list(vec![
                Expression::presence("Demo"),
                Expression::binding("name", self.name.serialize()),
                Expression::binding("addresses", self.addresses.serialize())
            ])
        }

        fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
        where
            Self: Sized,
        {
            let source_text = source_text.as_ref();
            exprs.eat_presence_if_present_and_matching("Demo");

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
                Address::Index(3, -1),
            ],
        }
    }

    const EXPECTED: &'static str = "(Demo name = cat addresses = (None (IpV4 (127.0.0.1)) (Index (3 -1))))";
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