use std::marker::PhantomData;
use easy_shape::{Fields, HasShape, NamedField, Shape, Struct};
use crate::cursor::Cursor;
use crate::error::Error;
use crate::interner::{InternedString, Interner};
use crate::span::Span;
use crate::value::Value;
use crate::token::{Token, TokenKind as Tk};

pub struct Parser<'a, T> {
    source: &'a str,
    cursor: Cursor<'a>,
    phantom: PhantomData<T>,
    interner: Interner,
}

impl<'a, T: HasShape> Parser<'a, T> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: Cursor::new(source),
            phantom: Default::default(),
            interner: Interner::new(),
        }
    }

    fn not_close_paren(&mut self) -> bool {
        !matches!(self.cursor.peek(), Some(Ok(t)) if t.kind == Tk::RightParen)
            && !self.cursor.finished()
    }

    fn intern_token(&mut self, token: Token) -> InternedString {
        self.interner.intern(token.resolve(self.source))
    }

    fn parse_binding(&mut self, val_shape: Shape) -> Result<Value, Error> {
        let key_token = self.cursor.expect(&[Tk::Text])?;
        let key = self.intern_token(key_token);
        self.cursor.expect(&[Tk::Equals])?;
        let val = self.parse_expr(val_shape)?;
        Ok(Value::binding(key, val, None))
    }

    fn parse_named_field(&mut self, fields: &[NamedField]) -> Result<(usize, Value), Error> {
        let key_token = self.cursor.expect(&[Tk::Text])?;
        let key_str = key_token.resolve(self.source);
        self.cursor.expect(&[Tk::Equals])?;

        let (idx, field) = fields.iter()
            .enumerate()
            .find(|(_, f)| f.name == key_str)
            .ok_or_else(|| Error::UnknownField {
                name: key_str.to_string(),
                span: key_token.span,
            })?;

        let key = self.intern_token(key_token);
        let val = self.parse_expr(field.shape)?;
        Ok((idx, Value::binding(key, val, None)))
    }

    fn fill_missing_fields(&mut self, fields: &[NamedField], seen: &[bool], open_span: Span) -> Result<Vec<Value>, Error> {
        let mut extra = Vec::new();
        for (field, _) in fields.iter().zip(seen).filter(|(_, seen)| !**seen) {
            let Shape::Option(_) = field.shape else {
                return Err(Error::MissingField {
                    name: field.name,
                    span: open_span,
                });
            };
            let key = self.interner.intern(field.name);
            let none = Value::presence(self.interner.intern("None"), None);
            extra.push(Value::binding(key, none, None));
        }
        Ok(extra)
    }

    fn parse_named_fields(&mut self, fields: &[NamedField], open_span: Span) -> Result<Vec<Value>, Error> {
        let mut values = Vec::new();
        let mut seen = vec![false; fields.len()];

        while self.not_close_paren() {
            let (idx, value) = self.parse_named_field(fields)?;
            seen[idx] = true;
            values.push(value);
        }

        values.extend(self.fill_missing_fields(fields, &seen, open_span)?);
        Ok(values)
    }

    fn parse_parens<F, I>(&mut self, name: impl Into<Option<InternedString>>, f: F) -> Result<Value, Error>
    where
        F: FnOnce(&mut Self) -> Result<I, Error>,
        I: IntoIterator<Item = Value>,
    {
        let open = self.cursor.expect(&[Tk::LeftParen])?;
        let values = f(self)?;
        let close = self.cursor.expect(&[Tk::RightParen])?;
        Ok(Value::list(name, values, None).with_span(open.span.merge(close.span)))
    }

    fn parse_option_none(&mut self) -> Result<Value, Error> {
        let none_token = self.cursor.expect(&[Tk::Text])?;
        let none = self.intern_token(none_token);

        if self.cursor.check(&[Tk::LeftParen]).is_ok() {
            return self.parse_parens(none, |_| Ok([]));
        }

        Ok(Value::presence(none, None).with_span(none_token.span))
    }

    fn parse_option_some(&mut self, inner: Shape) -> Result<Value, Error> {
        let some_token = self.cursor.expect(&[Tk::Text])?;
        let some = self.intern_token(some_token);
        self.parse_parens(some, |p| Ok([p.parse_expr(inner)?]))
    }

    fn parse_option(&mut self, inner: Shape) -> Result<Value, Error> {
        if self.cursor.check(&[Tk::LeftParen]).is_ok() {
            return self.parse_unit(); // () => None
        }
        if let Ok(token) = self.cursor.check(&[Tk::Text]) {
            match token.resolve(self.source) {
                "None" => return self.parse_option_none(),
                "Some" => return self.parse_option_some(inner),
                _ => {}
            }
        }
        // stretch goal
        let some = self.interner.intern("Some");
        let inner_value = self.parse_expr(inner)?;
        let span = inner_value.span.unwrap();
        Ok(Value::list(some, [inner_value], None).with_span(span))
    }

    fn parse_unit(&mut self) -> Result<Value, Error> {
        self.parse_parens(None, |_| Ok([]))
    }

    fn parse_sequence(&mut self, inner: Shape) -> Result<Value, Error> {
        self.parse_parens(None, |p| {
            let mut values = Vec::new();
            while p.not_close_paren() {
                values.push(p.parse_expr(inner)?);
            }
            Ok(values)
        })
    }

    fn parse_tuple(&mut self, shapes: &[Shape]) -> Result<Value, Error> {
        self.parse_parens(None, |p| {
            shapes.iter().map(|s| p.parse_expr(*s)).collect::<Result<Vec<_>, _>>()
        })
    }

    fn parse_map(&mut self, val_shape: Shape) -> Result<Value, Error> {
        self.parse_parens(None, |p| {
            let mut values = Vec::new();
            while p.not_close_paren() {
                values.push(p.parse_binding(val_shape)?);
            }
            Ok(values)
        })
    }

    fn parse_struct_fields(&mut self, name: InternedString, fields: &Fields) -> Result<Value, Error> {
        match fields {
            Fields::Unit => {
                self.parse_parens(name, |_| Ok([]))
            }
            Fields::NewType(inner) => {
                self.parse_parens(name, |p| Ok([p.parse_expr(**inner)?]))
            }
            Fields::Tuple(shapes) => {
                self.parse_parens(name, |p| {
                    shapes.iter().map(|s| p.parse_expr(*s)).collect::<Result<Vec<_>, _>>()
                })
            }
            Fields::Named(named_fields) => {
                let open = self.cursor.expect(&[Tk::LeftParen])?;
                let values = self.parse_named_fields(named_fields, open.span)?;
                let close = self.cursor.expect(&[Tk::RightParen])?;
                Ok(Value::list(name, values, None).with_span(open.span.merge(close.span)))
            }
        }
    }

    fn parse_struct(&mut self, struct_shape: &easy_shape::Struct) -> Result<Value, Error> {
        if let Ok(token) = self.cursor.check(&[Tk::Text]) {
            if token.resolve(self.source) == struct_shape.name {
                self.cursor.expect(&[Tk::Text])?;
            }
        }

        let name = self.interner.intern(struct_shape.name);
        self.parse_struct_fields(name, &struct_shape.fields)
    }

    fn parse_enum(&mut self, enum_shape: &easy_shape::Enum) -> Result<Value, Error> {
        let variant_token = self.cursor.expect(&[Tk::Text])?;
        let variant_name = variant_token.resolve(self.source);

        let variant = enum_shape.variants.iter()
            .find(|v| v.name == variant_name)
            .ok_or_else(|| Error::UnknownVariant {
                name: variant_name.to_string(),
                span: variant_token.span,
            })?;

        let interned_name = self.intern_token(variant_token);

        match &variant.fields {
            Fields::Unit => {
                Ok(Value::presence(interned_name, None).with_span(variant_token.span))
            }
            Fields::NewType(inner) => {
                if self.cursor.check(&[Tk::LeftParen]).is_ok() {
                    let v = self.parse_parens(interned_name, |p| Ok([p.parse_expr(**inner)?]))?;
                    let span = variant_token.span.merge(v.span.unwrap());
                    Ok(v.with_span(span))
                } else {
                    let inner_value = self.parse_expr(**inner)?;
                    let span = variant_token.span.merge(inner_value.span.unwrap());
                    Ok(Value::list(interned_name, [inner_value], None).with_span(span))
                }
            }
            _ => self.parse_struct_fields(interned_name, &variant.fields),
        }
    }

    fn parse_expr(&mut self, shape: Shape) -> Result<Value, Error> {
        match shape {
            Shape::Bool |
            Shape::I8 | Shape::I16 | Shape::I32 | Shape::I64 |
            Shape::U8 | Shape::U16 | Shape::U32 | Shape::U64 |
            Shape::F32 | Shape::F64 |
            Shape::Char => {
                let text = self.cursor.expect(&[Tk::Text])?;
                let interned = self.intern_token(text);
                Ok(Value::presence(interned, None).with_span(text.span))
            }
            Shape::String => {
                if self.cursor.check(&[Tk::LeftParen]).is_ok() {
                    return self.parse_sequence(Shape::String);
                }
                let text = self.cursor.expect(&[Tk::Text])?;
                let interned = self.intern_token(text);
                Ok(Value::presence(interned, None).with_span(text.span))
            }
            Shape::Option(inner) => self.parse_option(*inner),
            Shape::Unit => self.parse_unit(),
            Shape::Sequence(inner) => self.parse_sequence(*inner),
            Shape::Tuple(shapes) => self.parse_tuple(shapes),
            Shape::Map(_, v) => self.parse_map(*v),
            Shape::Struct(s) => self.parse_struct(s),
            Shape::Enum(e) => self.parse_enum(e),
        }
    }

    fn parse_top_level_struct(&mut self, shape: Struct) -> Result<Value, Error> {
        match shape.fields {
            Fields::Named(fields) => {
                let mut values = Vec::new();
                let mut seen = vec![false; fields.len()];

                while !self.cursor.finished() {
                    let (idx, value) = self.parse_named_field(fields)?;
                    seen[idx] = true;
                    values.push(value);
                }

                values.extend(self.fill_missing_fields(
                    fields,
                    &seen,
                    Span::new(0, self.source.len()),
                )?);

                let name = self.interner.intern(shape.name);
                Ok(Value::list(name, values, None))
            }
            _ => self.parse_expr(T::SHAPE)
        }
    }

    pub fn parse(mut self) -> Result<(Value, Interner), Error> {
        let value = match T::SHAPE {
            Shape::Struct(s) => self.parse_top_level_struct(*s)?,
            _ => self.parse_expr(T::SHAPE)?
        };
        Ok((value, self.interner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use easy_shape::HasShape;
        use crate::interner::Interner;
        use crate::parser::Parser;
        use crate::value::{Value, ValueKind};

        fn parse<T: HasShape>(source: &str) -> Value {
            Parser::<T>::new(source).parse().unwrap().0
        }

        fn parse_dump<T: HasShape>(source: &str) -> String {
            parse::<T>(source).dump()
        }

        // Primitives
        #[test]
        fn test_parse_bool_true() {
            assert_eq!(parse_dump::<bool>("true"), "true");
        }

        #[test]
        fn test_parse_bool_false() {
            assert_eq!(parse_dump::<bool>("false"), "false");
        }

        #[test]
        fn test_parse_u8() {
            assert_eq!(parse_dump::<u8>("255"), "255");
        }

        #[test]
        fn test_parse_u16() {
            assert_eq!(parse_dump::<u16>("65535"), "65535");
        }

        #[test]
        fn test_parse_u32() {
            assert_eq!(parse_dump::<u32>("42"), "42");
        }

        #[test]
        fn test_parse_u64() {
            assert_eq!(parse_dump::<u64>("1000000"), "1000000");
        }

        #[test]
        fn test_parse_i8() {
            assert_eq!(parse_dump::<i8>("-128"), "-128");
        }

        #[test]
        fn test_parse_i32() {
            assert_eq!(parse_dump::<i32>("-42"), "-42");
        }

        #[test]
        fn test_parse_f32() {
            assert_eq!(parse_dump::<f32>("3.14"), "3.14");
        }

        #[test]
        fn test_parse_f64() {
            assert_eq!(parse_dump::<f64>("-1.5"), "-1.5");
        }

        #[test]
        fn test_parse_char() {
            assert_eq!(parse_dump::<char>("a"), "a");
        }

        // Strings
        #[test]
        fn test_parse_string_single_word() {
            assert_eq!(parse_dump::<String>("hello"), "hello");
        }

        #[test]
        fn test_parse_string_multiple_words() {
            assert_eq!(parse_dump::<String>("(hello world)"), "(hello world)");
        }

        #[test]
        fn test_parse_string_empty() {
            assert_eq!(parse_dump::<String>("()"), "()");
        }

        // Unit
        #[test]
        fn test_parse_unit() {
            assert_eq!(parse_dump::<()>("()"), "()");
        }

        // Option
        #[test]
        fn test_parse_option_none_keyword() {
            assert_eq!(parse_dump::<Option<u32>>("None"), "None");
        }

        #[test]
        fn test_parse_option_none_empty_parens() {
            assert_eq!(parse_dump::<Option<u32>>("()"), "()");
        }

        #[test]
        fn test_parse_option_none_named_empty_parens() {
            assert_eq!(parse_dump::<Option<u32>>("None()"), "None()");
        }

        #[test]
        fn test_parse_option_some_explicit() {
            assert_eq!(parse_dump::<Option<u32>>("Some(42)"), "Some(42)");
        }

        #[test]
        fn test_parse_option_some_implicit() {
            // stretch goal: bare value treated as Some
            assert_eq!(parse_dump::<Option<u32>>("42"), "Some(42)");
        }

        #[test]
        fn test_parse_option_nested_none() {
            assert_eq!(parse_dump::<Option<Option<u32>>>("None"), "None");
        }

        #[test]
        fn test_parse_option_nested_some_none() {
            assert_eq!(parse_dump::<Option<Option<u32>>>("Some(None)"), "Some(None)");
        }

        #[test]
        fn test_parse_option_nested_some_some() {
            assert_eq!(parse_dump::<Option<Option<u32>>>("Some(Some(42))"), "Some(Some(42))");
        }

        // Vec / Sequence
        #[test]
        fn test_parse_vec_empty() {
            assert_eq!(parse_dump::<Vec<u32>>("()"), "()");
        }

        #[test]
        fn test_parse_vec_single() {
            assert_eq!(parse_dump::<Vec<u32>>("(1)"), "(1)");
        }

        #[test]
        fn test_parse_vec_multiple() {
            assert_eq!(parse_dump::<Vec<u32>>("(1 2 3 4)"), "(1 2 3 4)");
        }

        #[test]
        fn test_parse_vec_nested() {
            assert_eq!(parse_dump::<Vec<Vec<u32>>>("((1 2) (3 4))"), "((1 2) (3 4))");
        }

        #[test]
        fn test_parse_vec_of_options() {
            assert_eq!(parse_dump::<Vec<Option<u32>>>("(Some(1) None Some(3))"), "(Some(1) None Some(3))");
        }

        // Tuple
        #[test]
        fn test_parse_tuple_1() {
            assert_eq!(parse_dump::<(u32,)>("(42)"), "(42)");
        }

        #[test]
        fn test_parse_tuple_2() {
            assert_eq!(parse_dump::<(u32, bool)>("(42 true)"), "(42 true)");
        }

        #[test]
        fn test_parse_tuple_3() {
            assert_eq!(parse_dump::<(u32, bool, f32)>("(42 true 3.14)"), "(42 true 3.14)");
        }

        // Map
        #[test]
        fn test_parse_map_empty() {
            assert_eq!(parse_dump::<std::collections::HashMap<String, u32>>("()"), "()");
        }

        #[test]
        fn test_parse_map_single() {
            assert_eq!(parse_dump::<std::collections::HashMap<String, u32>>("(foo = 1)"), "(foo = 1)");
        }

        #[test]
        fn test_parse_btreemap_multiple() {
            // BTreeMap for deterministic ordering
            assert_eq!(
                parse_dump::<std::collections::BTreeMap<String, u32>>("(a = 1 b = 2 c = 3)"),
                "(a = 1 b = 2 c = 3)"
            );
        }

        #[test]
        fn test_parse_map_nested_value() {
            assert_eq!(
                parse_dump::<std::collections::BTreeMap<String, Vec<u32>>>("(a = (1 2) b = (3 4))"),
                "(a = (1 2) b = (3 4))"
            );
        }

        // Arrays
        #[test]
        fn test_parse_array_3() {
            assert_eq!(parse_dump::<[u32; 3]>("(1 2 3)"), "(1 2 3)");
        }

        #[test]
        fn test_parse_array_wrong_length() {
            use crate::from_value::FromValue;
            use crate::value::Value as V;
            let mut interner = crate::interner::Interner::new();
            let value = Parser::<Vec<u32>>::new("(1 2)").parse().unwrap().0;
            assert!(<[u32; 3]>::from_value(value, "").is_err());
        }

        // Comments
        #[test]
        fn test_parse_ignores_comments() {
            assert_eq!(parse_dump::<u32>("# this is a comment\n42"), "42");
        }

        #[test]
        fn test_parse_inline_comment() {
            assert_eq!(parse_dump::<Vec<u32>>("(1 # comment\n2 3)"), "(1 2 3)");
        }

        // Whitespace
        #[test]
        fn test_parse_extra_whitespace() {
            assert_eq!(parse_dump::<u32>("   42   "), "42");
        }

        #[test]
        fn test_parse_newlines() {
            assert_eq!(parse_dump::<Vec<u32>>("(\n1\n2\n3\n)"), "(1 2 3)");
        }

        // HashSet / BTreeSet
        #[test]
        fn test_parse_btreeset() {
            assert_eq!(
                parse_dump::<std::collections::BTreeSet<u32>>("(3 1 2)"),
                "(3 1 2)"
            );
        }
    }
}