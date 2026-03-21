use crate::error::Error;
use crate::interner::InternedString;
use crate::span::Span;

pub const PRESENCE_STR: &'static str = "PRESENCE";
pub const BINDING_STR: &'static str = "BINDING";
pub const LIST: &'static str = "LIST";

#[derive(Debug, Clone)]
pub enum ValueKind {
    Presence(InternedString),
    Binding(InternedString, Box<Value>),
    List(Option<InternedString>, Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub span: Option<Span>,
    pub comment: Option<InternedString>,
}

impl Value {
    pub fn new(kind: ValueKind, span: impl Into<Option<Span>>, comment:impl Into<Option<InternedString>>) -> Self {
        Self { kind, span: span.into(), comment: comment.into() }
    }
    
    pub fn presence(text: InternedString, comment:impl Into<Option<InternedString>>) -> Self {
        Self::new(ValueKind::Presence(text), None, comment)
    }

    pub fn binding(key: InternedString, value: Value, comment:impl Into<Option<InternedString>>) -> Self {
        Self::new(ValueKind::Binding(key, Box::new(value)), None, comment)
    }
    
    pub fn list(name: impl Into<Option<InternedString>>, values: impl IntoIterator<Item = Value>, comment:impl Into<Option<InternedString>>) -> Self {
        Self::new(ValueKind::List(name.into(), Vec::from_iter(values)), None, comment)
    }

    pub fn with_comment(mut self, comment: impl Into<Option<InternedString>>) -> Self {
        self.comment = comment.into();
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    
    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            ValueKind::Presence(_) => PRESENCE_STR,
            ValueKind::Binding(_, _) => BINDING_STR,
            ValueKind::List(_, _) => LIST,
        }
    }
    
    pub fn to_presence(self) -> Result<InternedString, Error> {
        match self.kind {
            ValueKind::Presence(text) => Ok(text),
            _ => Err(Error::WrongShape {
                expected: &[PRESENCE_STR],
                got: self.kind_str(),
                span: self.span.unwrap_or(Span::new(0, 0)),
            })
        }
    }

    pub fn to_binding(self) -> Result<(InternedString, Value), Error> {
        match self.kind {
            ValueKind::Binding(text, value) => Ok((text, *value)),
            _ => Err(Error::WrongShape {
                expected: &[BINDING_STR],
                got: self.kind_str(),
                span: self.span.unwrap(),
            })
        }
    }
    
    pub fn to_list(self) -> Result<(Option<InternedString>, Vec<Value>), Error> {
        match self.kind {
            ValueKind::List(text, list) => Ok((text, list)),
            _ => Err(Error::WrongShape {
                expected: &[BINDING_STR],
                got: self.kind_str(),
                span: self.span.unwrap(),
            })
        }
    }

    pub fn is_none_equivalent(&self) -> bool {
        match &self.kind {
            ValueKind::List(name, values) => {
                values.is_empty() && name.as_ref().map_or(true, |n| n.resolve() == "None")
            },
            ValueKind::Presence(text) => text.resolve() == "None",
            ValueKind::Binding(_, _) => false,
        }
    }

    pub fn dump(&self) -> String {
        let mut out = String::new();
        self.write_dump(&mut out, 0, false);
        out
    }

    pub fn pretty_dump(&self) -> String {
        let mut out = String::new();
        self.write_dump(&mut out, 0, true);
        out
    }

    fn write_dump(&self, out: &mut String, indent: usize, pretty: bool) {
        match &self.kind {
            ValueKind::Presence(text) => {
                out.push_str(text.resolve());
            }
            ValueKind::Binding(key, value) => {
                out.push_str(key.resolve());
                out.push_str(" = ");
                value.write_dump(out, indent, pretty);
            }
            ValueKind::List(name, values) => {
                if let Some(name) = name {
                    out.push_str(name.resolve());
                }
                if pretty && !values.is_empty() {
                    let inner_indent = indent + 1;
                    let indent_str = "    ".repeat(indent);
                    let inner_indent_str = "    ".repeat(inner_indent);
                    out.push('(');
                    out.push('\n');
                    for value in values {
                        out.push_str(&inner_indent_str);
                        value.write_dump(out, inner_indent, pretty);
                        out.push('\n');
                    }
                    out.push_str(&indent_str);
                    out.push(')');
                } else {
                    out.push('(');
                    for (i, value) in values.iter().enumerate() {
                        if i > 0 { out.push(' '); }
                        value.write_dump(out, indent, pretty);
                    }
                    out.push(')');
                }
            }
        }
    }
}