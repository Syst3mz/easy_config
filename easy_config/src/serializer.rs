use std::io;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use crate::error::TextualError;

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' => out.push_str(r"\("),
            ')' => out.push_str(r"\)"),
            '=' => out.push_str(r"\="),
            '\\' => out.push_str(r"\\"),
            '#' => out.push_str(r"\#"),
            _ => out.push(c),
        }
    }
    out
}

pub struct Serializer<W: io::Write> {
    output: W,
    pretty: bool,
    indent: usize,
    after_open: bool,
    is_empty: bool,
}

impl<W: io::Write> Serializer<W> {
    pub fn new(output: W) -> Self {
        Self { output, pretty: false, indent: 0, after_open: false, is_empty: true }
    }

    pub fn new_pretty(output: W) -> Self {
        Self { output, pretty: true, indent: 0, after_open: false, is_empty: true }
    }

    fn push(&mut self, s: &str) -> io::Result<()> {
        if !self.is_empty && !self.after_open {
            if self.pretty {
                write!(self.output, "\n")?;
                for _ in 0..self.indent {
                    write!(self.output, "    ")?;
                }
            } else {
                write!(self.output, " ")?;
            }
        }
        self.after_open = false;
        self.is_empty = false;
        write!(self.output, "{s}")
    }

    fn open_paren(&mut self) -> io::Result<()> {
        if !self.is_empty && !self.after_open {
            write!(self.output, " ")?;
        }
        write!(self.output, "(")?;
        self.indent += 1;
        self.after_open = true;
        self.is_empty = false;
        Ok(())
    }

    fn close_paren(&mut self) -> io::Result<()> {
        self.indent -= 1;
        self.after_open = false;
        if self.pretty {
            write!(self.output, "\n")?;
            for _ in 0..self.indent {
                write!(self.output, "    ")?;
            }
        }
        write!(self.output, ")")
    }
}

pub fn to_writer<W: io::Write, T: serde::Serialize>(
    writer: W, value: &T
) -> Result<(), TextualError> {
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

pub fn to_writer_pretty<W: io::Write, T: serde::Serialize>(
    writer: W, value: &T
) -> Result<(), TextualError> {
    let mut ser = Serializer::new_pretty(writer);
    value.serialize(&mut ser)
}

pub fn to_string<T: serde::Serialize>(value: &T) -> Result<String, TextualError> {
    let mut buf = Vec::new();
    to_writer(&mut buf, value)?;
    Ok(String::from_utf8(buf).unwrap())
}

pub fn to_string_pretty<T: serde::Serialize>(value: &T) -> Result<String, TextualError> {
    let mut buf = Vec::new();
    to_writer_pretty(&mut buf, value)?;
    Ok(String::from_utf8(buf).unwrap())
}

impl serde::ser::Error for TextualError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        TextualError {
            message: msg.to_string(),
            location: None,
        }
    }
}

impl<'a, W: io::Write> serde::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = TextualError;

    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = SeqSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = MapSerializer<'a>;
    type SerializeStructVariant = MapSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<(), Self::Error> {
        self.push(if v { "true" } else { "false" })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_i64(self, v: i64) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_u64(self, v: u64) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_f64(self, v: f64) -> Result<(), Self::Error> {
        Ok(self.push(&v.to_string())?)
    }
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        Ok(self.push(&escape(&v.to_string()))?)
    }
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        Ok(self.push(&escape(&v.to_string()))?)
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        Ok(self.push("None")?)
    }

    fn serialize_some<T: serde::Serialize + ?Sized>(self, value: &T) -> Result<(), Self::Error> {
        self.push("Some(")?;
        value.serialize(&mut *self)?;
        self.output.push(')');
        Ok(())
    }

    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.push("()");
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Self::Error> {
        Ok(self.push(name)?)
    }

    fn serialize_unit_variant(
        self, _name: &'static str, _idx: u32, variant: &'static str
    ) -> Result<(), Self::Error> {
        Ok(self.push(variant)?)
    }

    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self, _name: &'static str, value: &T
    ) -> Result<(), Self::Error> {
        Ok(value.serialize(self)?)
    }

    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self, _name: &'static str, _idx: u32, variant: &'static str, value: &T
    ) -> Result<(), Self::Error> {
        self.push(&format!("{variant}("))?;
        value.serialize(&mut *self)?;
        self.output.push(')');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output.push_str(if self.output.is_empty() { "(" } else { " (" });
        Ok(SeqSerializer { ser: self, first: true })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self, _name: &'static str, len: usize
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self, _name: &'static str, _idx: u32, variant: &'static str, len: usize
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.push(&format!("{variant}"));
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.output.push_str(if self.output.is_empty() { "(" } else { " (" });
        Ok(MapSerializer { ser: self, first: true })
    }

    fn serialize_struct(
        self, _name: &'static str, len: usize
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self, _name: &'static str, _idx: u32, variant: &'static str, len: usize
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.push(variant);
        self.serialize_map(Some(len))
    }
}

// --- seq/tuple serializer ---

pub struct SeqSerializer<'a> {
    ser: &'a mut Serializer,
    first: bool,
}

impl<'a> SeqSerializer<'a> {
    fn serialize_element_inner<T: serde::Serialize + ?Sized>(
        &mut self, value: &T
    ) -> Result<(), TextualError> {
        if !self.first {
            // push will add spacing via the normal path
        }
        self.first = false;
        value.serialize(&mut *self.ser)
    }
}

impl<'a> SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = TextualError;
    fn serialize_element<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_element_inner(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeTuple for SeqSerializer<'a> {
    type Ok = ();
    type Error = TextualError;
    fn serialize_element<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_element_inner(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = ();
    type Error = TextualError;
    fn serialize_field<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_element_inner(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = ();
    type Error = TextualError;
    fn serialize_field<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_element_inner(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

// --- map/struct serializer ---

pub struct MapSerializer<'a> {
    ser: &'a mut Serializer,
    first: bool,
}

impl<'a> SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = TextualError;

    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.first = false;
        key.serialize(&mut *self.ser)
    }

    fn serialize_value<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.ser.output.push('=');
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeStruct for MapSerializer<'a> {
    type Ok = ();
    type Error = TextualError;

    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self, key: &'static str, value: &T
    ) -> Result<(), Self::Error> {
        // spacing before each field
        if !self.ser.output.ends_with('(') {
            self.ser.output.push(' ');
        }
        self.ser.output.push_str(key);
        self.ser.output.push('=');
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeStructVariant for MapSerializer<'a> {
    type Ok = ();
    type Error = TextualError;

    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self, key: &'static str, value: &T
    ) -> Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::deserializer::from_str;
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Point {
        x: f64,
        y: f64
    }

    #[test]
    fn roundtrip_struct() {
        let original = Point { x: 1.0, y: 2.0 };
        let serialized = to_string(&original).unwrap();
        let deserialized: Point = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Shape {
        Empty,
        Circle(f64),
        Rect { w: f64, h: f64 },
    }

    #[test]
    fn roundtrip_enum() {
        for shape in [Shape::Empty, Shape::Circle(5.0), Shape::Rect { w: 3.0, h: 4.0 }] {
            let s = to_string(&shape).unwrap();
            assert_eq!(shape, from_str::<Shape>(&s).unwrap());
        }
    }
}