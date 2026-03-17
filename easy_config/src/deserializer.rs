use std::fmt::Display;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserializer as SerdeDeserializer;
use crate::cursor::Cursor;
use crate::error::{Error, TextualError, ToTextualErrorResult, WithContext};
use crate::token::{Token, TokenKind as Tk};

pub struct Deserializer<'de> {
    tokens: Cursor<'de>,
    source: &'de str,
}

impl<'de> Deserializer<'de> {
    fn peek(&'_ mut self) -> Result<Token<'_>, TextualError> {
        self.tokens.peek().unwrap_or(Err(Error::ReachedEoi)).to_textual_error(self.source)
    }
    fn check(&'_ mut self, expected: &'static [Tk]) -> Result<Token<'_>, TextualError> {
        self.tokens.check(&expected).to_textual_error(self.source)
    }

    fn expect(&mut self, expected: &'static [Tk]) -> Result<Token<'de>, TextualError> {
        self.tokens.expect(expected).to_textual_error(self.source)
    }

    fn consume_name_if_present(&mut self, name: &str) -> Result<(), Error> {
        if let Ok(token) = self.expect(&[Tk::Text]) {
            if name != token.lexeme {
                return Err(Error::unable_to_convert_token_to(token, name))
            }
        }

        Ok(())
    }

    fn unescape(text: impl AsRef<str>) -> String {
        let mut out = String::with_capacity(text.as_ref().len());
        let mut chars = text.as_ref().chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    fn consume_string(&mut self) -> Result<String, TextualError> {
        if let Ok(token) = self.expect(&[Tk::Text]) {
            return Ok(Self::unescape(token.lexeme.to_string()))
        }

        let _left_paren = self.expect(&[Tk::LeftParen])?;

        let mut span = self.peek()?.span;

        loop {
            if self.check(&[Tk::RightParen]).is_ok() { break }
            let token = self.expect(&[Tk::Text])?;
            span = span.merge(token.span);
        }

        let _right_paren = self.expect(&[Tk::RightParen])?;

        Ok(Self::unescape(self.source[span.to_range()].to_string()))
    }

    pub fn new(source: &'de str) -> Deserializer<'de> {
        Self {
            tokens: Cursor::new(source),
            source,
        }
    }
}

impl serde::de::Error for TextualError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display
    {
        TextualError {
            message: msg.to_string(),
            location: None,
        }
    }
}

macro_rules! deserialize_primitive {
    ($self:ident, $typ:ty) => {{
        let has_paren = $self.check(&[Tk::LeftParen]).is_ok();
        if has_paren { $self.expect(&[Tk::LeftParen])?; }
        let token = $self.expect(&[Tk::Text])?;
        let value = token.lexeme.parse::<$typ>()
            .map_err(|_| Error::unable_to_convert_token_to(token, stringify!($typ))
                .to_textual_error($self.source))?;
        if has_paren { $self.expect(&[Tk::RightParen])?; }
        value
    }};
}

impl<'de> SeqAccess<'de> for &mut Deserializer<'de> {
    type Error = TextualError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>
    {
        if self.check(&[Tk::RightParen]).is_ok() {
            return Ok(None);
        }

        seed.deserialize(&mut **self).map(Some)
    }
}

impl<'de> MapAccess<'de> for &mut Deserializer<'de> {
    type Error = TextualError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>
    {
        if self.check(&[Tk::RightParen]).is_ok() {
            return Ok(None);
        }
        seed.deserialize(&mut **self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>
    {
        self.expect(&[Tk::Equals])?;
        seed.deserialize(&mut **self)
    }
}

impl<'de> serde::de::EnumAccess<'de> for &mut Deserializer<'de> {
    type Error = TextualError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>
    {
        let tag = seed.deserialize(&mut *self)?;
        Ok((tag, self))
    }
}

impl<'de> serde::de::VariantAccess<'de> for &mut Deserializer<'de> {
    type Error = TextualError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        let Ok(_left_paren) = self.expect(&[Tk::LeftParen]) else {return Ok(())};
        let _right_paren = self.expect(&[Tk::RightParen])?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>
    {
        self.expect(&[Tk::LeftParen])?;
        let value = seed.deserialize(&mut *self)?;
        self.expect(&[Tk::RightParen])?;
        Ok(value)
    }

    fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_map(visitor)
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = TextualError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let token = self.expect(&[Tk::Text])?;
        let kind = match token.lexeme.to_lowercase().as_ref() {
            "true" => true,
            "false" => false,
            _ => return Err(
                Error::unable_to_convert_token_to(token, "bool")
                    .with_context("Bools must have be exactly the text \"true\" or \"false\" (case insensitive)")
                    .to_textual_error(self.source)
            )
        };

        visitor.visit_bool(kind)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_i8(deserialize_primitive!(self, i8))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_i16(deserialize_primitive!(self, i16))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_i32(deserialize_primitive!(self, i32))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_i64(deserialize_primitive!(self, i64))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_u8(deserialize_primitive!(self, u8))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_u16(deserialize_primitive!(self, u16))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_u32(deserialize_primitive!(self, u32))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_u64(deserialize_primitive!(self, u64))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_f32(deserialize_primitive!(self, f32))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_f64(deserialize_primitive!(self, f64))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let token = self.expect(&[Tk::Text])?;
        let mut chars = token.lexeme.chars();
        let Some(c) = chars.next() else { unreachable!() };
        if chars.next().is_some() {
            return Err(Error::unable_to_convert_token_to(token, "char").to_textual_error(self.source));
        }

        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_str(&self.consume_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_string(self.consume_string()?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let discriminant = self.expect(&[Tk::Text])?;
        match discriminant.lexeme{
            "None" => return visitor.visit_none(),
            "Some" => {},
            _ => return Err(
                Error::unable_to_convert_token_to(discriminant, "Option")
                    .with_context("Option discriminant must be either \"Some\" or \"None\"")
                    .to_textual_error(self.source)
            ),
        }

        let _left_paren = self.expect(&[Tk::LeftParen])?;
        let value = visitor.visit_some(&mut *self)?;
        let _right_paren = self.expect(&[Tk::RightParen])?;

        Ok(value)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let _left_paren = self.expect(&[Tk::LeftParen])?;
        let _right_paren = self.expect(&[Tk::RightParen])?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let err_text = format!("Expected \"{}\" or \"()\"", name);
        if self.check(&[Tk::Text]).is_ok() {
            self.consume_name_if_present(name)
                .with_context(err_text)
                .to_textual_error(self.source)?;
            return visitor.visit_unit();
        }

        let _left_paren = self.tokens
            .expect(&[Tk::LeftParen])
            .with_context(&err_text)
            .to_textual_error(self.source)?;

        let _right_paren = self.tokens
            .expect(&[Tk::RightParen])
            .with_context(&err_text)
            .to_textual_error(self.source)?;

        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.consume_name_if_present(name)
            .with_context(format!("Expected \"{}\"", name))
            .to_textual_error(self.source)?;

        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let _left_paren = self.expect(&[Tk::LeftParen])?;
        let value = visitor.visit_seq(&mut *self);
        let _right_paren = self.expect(&[Tk::RightParen])?;

        value
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.consume_name_if_present(name).to_textual_error(self.source)?;
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let _left_paren = self.expect(&[Tk::LeftParen])?;
        let value = visitor.visit_map(&mut *self);
        let _right_paren = self.expect(&[Tk::RightParen])?;

        value
    }

    fn deserialize_struct<V>(self, name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.consume_name_if_present(name).to_textual_error(self.source)?;
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        visitor.visit_enum(&mut *self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        match self.peek()?.kind {
            // consumes a nested list
            Tk::LeftParen => {
                self.expect(&[Tk::LeftParen])?;
                while self.check(&[Tk::RightParen]).is_err() {
                    self.deserialize_ignored_any(serde::de::IgnoredAny)?;
                }
                self.expect(&[Tk::RightParen])?;
            }
            _ => {
                self.expect(&[Tk::Text])?;
            }
        }
        visitor.visit_unit()
    }
}

pub fn from_str<'de, T: serde::Deserialize<'de>>(input: &'de str) -> Result<T, TextualError> {
    let mut de = Deserializer::new(input);
    T::deserialize(&mut de)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use serde::Deserialize;
    use super::*;

    fn de<'de, T: Deserialize<'de>>(input: &'de str) -> T {
        from_str(input).expect("deserialization failed")
    }

    fn de_err<'de, T: Deserialize<'de>+Debug>(input: &'de str) -> TextualError {
        from_str::<T>(input).expect_err("expected error but deserialization succeeded")
    }

    #[test]
    fn bool_true() {
        assert_eq!(de::<bool>("true"), true);
    }

    #[test]
    fn bool_false() {
        assert_eq!(de::<bool>("false"), false);
    }

    #[test]
    fn bool_case_insensitive() {
        assert_eq!(de::<bool>("True"), true);
        assert_eq!(de::<bool>("FALSE"), false);
    }

    #[test]
    fn bool_invalid_is_error() {
        de_err::<bool>("yes");
    }

    #[test]
    fn integers() {
        assert_eq!(de::<i8>("-128"), -128i8);
        assert_eq!(de::<i8>("127"), 127i8);
        assert_eq!(de::<u8>("255"), 255u8);
        assert_eq!(de::<i32>("-2147483648"), i32::MIN);
        assert_eq!(de::<u64>("18446744073709551615"), u64::MAX);
    }

    #[test]
    fn integer_overflow_is_error() {
        de_err::<u8>("256");
        de_err::<i8>("-129");
    }

    #[test]
    fn floats() {
        assert_eq!(de::<f32>("1.5"), 1.5f32);
        assert_eq!(de::<f64>("3.14"), 3.14f64);
        assert_eq!(de::<f64>("-0.5"), -0.5f64);
    }

    #[test]
    fn char_single() {
        assert_eq!(de::<char>("a"), 'a');
    }

    #[test]
    fn char_multi_is_error() {
        de_err::<char>("ab");
    }

    // --- trailing tokens are ignored ---

    #[test]
    fn trailing_tokens_ignored() {
        assert_eq!(de::<u8>("42 garbage"), 42u8);
        assert_eq!(de::<bool>("true false"), true);
    }

    #[test]
    fn trailing_parens_ignored() {
        assert_eq!(de::<u8>("42 (extra stuff)"), 42u8);
    }

    // --- strings and unescaping ---

    #[test]
    fn plain_string() {
        assert_eq!(de::<String>("hello"), "hello");
    }

    #[test]
    fn escaped_paren_in_string() {
        assert_eq!(de::<String>(r"hello\(world"), "hello(world");
    }

    #[test]
    fn escaped_close_paren_in_string() {
        assert_eq!(de::<String>(r"hello\)world"), "hello)world");
    }

    #[test]
    fn escaped_equals_in_string() {
        assert_eq!(de::<String>(r"hello\=world"), "hello=world");
    }

    #[test]
    fn escaped_backslash_in_string() {
        assert_eq!(de::<String>(r"hello\\world"), r"hello\world");
    }

    #[test]
    fn multiple_escapes_in_string() {
        assert_eq!(de::<String>(r"a\(b\)c"), "a(b)c");
    }

    #[test]
    fn string_with_no_escapes_is_unchanged() {
        assert_eq!(de::<String>("hello"), "hello");
    }

    // --- option ---

    #[test]
    fn option_none() {
        assert_eq!(de::<Option<u8>>("None"), None);
    }

    #[test]
    fn option_some() {
        assert_eq!(de::<Option<u8>>("Some(42)"), Some(42u8));
    }

    #[test]
    fn option_some_string_unescaped() {
        assert_eq!(de::<Option<String>>(r"Some(hello\(world)"), Some("hello(world".to_string()));
    }

    #[test]
    fn option_invalid_discriminant_is_error() {
        de_err::<Option<u8>>("maybe(42)");
    }

    #[test]
    fn nested_option() {
        assert_eq!(de::<Option<Option<u8>>>("Some(Some(1))"), Some(Some(1u8)));
        assert_eq!(de::<Option<Option<u8>>>("Some(None)"), Some(None));
    }

    // --- unit ---

    #[test]
    fn unit() {
        assert_eq!(de::<()>("()"), ());
    }

    // --- sequences ---

    #[test]
    fn vec_empty() {
        assert_eq!(de::<Vec<u8>>("()"), vec![]);
    }

    #[test]
    fn vec_single() {
        assert_eq!(de::<Vec<u8>>("(1)"), vec![1u8]);
    }

    #[test]
    fn vec_multiple() {
        assert_eq!(de::<Vec<u8>>("(1 2 3)"), vec![1u8, 2, 3]);
    }

    #[test]
    fn nested_vec() {
        assert_eq!(
            de::<Vec<Vec<u8>>>("((1 2) (3 4))"),
            vec![vec![1u8, 2], vec![3, 4]]
        );
    }

    #[test]
    fn vec_of_strings_unescaped() {
        assert_eq!(
            de::<Vec<String>>(r"(a\(b c\)d)"),
            vec!["a(b".to_string(), "c)d".to_string()]
        );
    }

    // --- maps ---

    #[test]
    fn hashmap_empty() {
        assert_eq!(de::<HashMap<String, u8>>("()"), HashMap::new());
    }

    #[test]
    fn hashmap_single() {
        let map = de::<HashMap<String, u8>>("(a=1)");
        assert_eq!(map["a"], 1);
    }

    #[test]
    fn hashmap_multiple() {
        let map = de::<HashMap<String, u8>>("(a=1 b=2 c=3)");
        assert_eq!(map["a"], 1);
        assert_eq!(map["b"], 2);
        assert_eq!(map["c"], 3);
    }

    #[test]
    fn hashmap_key_unescaped() {
        let map = de::<HashMap<String, u8>>(r"(a\=b=1)");
        assert_eq!(map["a=b"], 1);
    }

    // --- named structs ---

    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: f32,
        y: f32,
    }

    #[test]
    fn named_struct_with_tag() {
        assert_eq!(de::<Point>("Point(x=1.0 y=2.0)"), Point { x: 1.0, y: 2.0 });
    }

    #[test]
    fn named_struct_without_tag() {
        assert_eq!(de::<Point>("(x=1.0 y=2.0)"), Point { x: 1.0, y: 2.0 });
    }

    #[test]
    fn named_struct_fields_out_of_order() {
        assert_eq!(de::<Point>("(y=2.0 x=1.0)"), Point { x: 1.0, y: 2.0 });
    }

    #[test]
    fn named_struct_wrong_tag_is_error() {
        de_err::<Point>("NotPoint(x=1.0 y=2.0)");
    }

    // --- tuple structs ---

    #[derive(Deserialize, Debug, PartialEq)]
    struct Pair(u8, u8);

    #[test]
    fn tuple_struct_with_tag() {
        assert_eq!(de::<Pair>("Pair(1 2)"), Pair(1, 2));
    }

    #[test]
    fn tuple_struct_without_tag() {
        assert_eq!(de::<Pair>("(1 2)"), Pair(1, 2));
    }

    #[test]
    fn tuple_struct_wrong_tag_is_error() {
        de_err::<Pair>("NotPair(1 2)");
    }

    // --- newtype structs ---

    #[derive(Deserialize, Debug, PartialEq)]
    struct Wrapper(u32);

    #[test]
    fn newtype_struct_with_tag() {
        assert_eq!(de::<Wrapper>("Wrapper(42)"), Wrapper(42));
    }

    #[test]
    fn newtype_struct_without_tag() {
        assert_eq!(de::<Wrapper>("(42)"), Wrapper(42));
    }

    #[test]
    fn newtype_struct_wrong_tag_is_error() {
        de_err::<Wrapper>("NotWrapper(42)");
    }

    // --- unit structs ---

    #[derive(Deserialize, Debug, PartialEq)]
    struct Unit;

    #[test]
    fn unit_struct_with_tag() {
        assert_eq!(de::<Unit>("Unit"), Unit);
    }

    #[test]
    fn unit_struct_with_parens() {
        assert_eq!(de::<Unit>("()"), Unit);
    }

    #[test]
    fn unit_struct_wrong_tag_is_error() {
        de_err::<Unit>("NotUnit");
    }

    // --- enums ---

    #[derive(Deserialize, Debug, PartialEq)]
    enum Shape {
        Circle(f32),
        Rect { w: f32, h: f32 },
        Empty,
    }

    #[test]
    fn enum_unit_variant() {
        assert_eq!(de::<Shape>("Empty"), Shape::Empty);
    }

    #[test]
    fn enum_newtype_variant() {
        assert_eq!(de::<Shape>("Circle(5.0)"), Shape::Circle(5.0));
    }

    #[test]
    fn enum_struct_variant() {
        assert_eq!(
            de::<Shape>("Rect(w=3.0 h=4.0)"),
            Shape::Rect { w: 3.0, h: 4.0 }
        );
    }

    #[test]
    fn enum_unknown_variant_is_error() {
        de_err::<Shape>("Triangle");
    }

    // --- complex / nested ---

    #[derive(Deserialize, Debug, PartialEq)]
    struct Points(Vec<Point>);

    #[test]
    fn newtype_wrapping_vec_of_named_structs() {
        assert_eq!(
            de::<Points>("((x=1.0 y=2.0) (x=3.0 y=4.0))"),
            Points(vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }])
        );
    }

    #[test]
    fn newtype_wrapping_vec_with_tags() {
        assert_eq!(
            de::<Points>("(Point(x=1.0 y=2.0) Point(x=3.0 y=4.0))"),
            Points(vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }])
        );
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        value: u32,
        enabled: bool,
    }

    #[test]
    fn realistic_config() {
        assert_eq!(
            de::<Config>("(name=foo value=42 enabled=true)"),
            Config { name: "foo".to_string(), value: 42, enabled: true }
        );
    }

    #[test]
    fn realistic_config_with_escaped_name() {
        assert_eq!(
            de::<Config>(r"(name=hello\=world value=1 enabled=false)"),
            Config { name: "hello=world".to_string(), value: 1, enabled: false }
        );
    }

    #[test]
    fn vec_of_enums() {
        assert_eq!(
            de::<Vec<Shape>>("(Empty Circle(1.0) Empty)"),
            vec![Shape::Empty, Shape::Circle(1.0), Shape::Empty]
        );
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            de::<Vec<Option<Vec<u8>>>>("(Some((1 2)) None Some((3)))"),
            vec![Some(vec![1u8, 2]), None, Some(vec![3])]
        );
    }
}