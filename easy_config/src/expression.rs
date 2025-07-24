use std::fmt::Display;
use crate::expression::ExpressionData::{List, Binding, Presence};
use crate::lexical_range::LexicalSpan;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Atom {
    Text(String),
    Number(String),
}

impl Atom {
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Atom::Text(s) | Atom::Number(s) => s
        })
    }
}
impl From<String> for Atom {
    fn from(s: String) -> Self {
        Atom::Text(s)
    }
}
impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom::Text(s.to_string())
    }
}

macro_rules! numeric_into_atom {
    ($ty:ty) => {
        impl From<$ty> for Atom {
            fn from(value: $ty) -> Atom {
                Atom::Number(value.to_string())
            }
        }
    };
}

numeric_into_atom!(u8);
numeric_into_atom!(u16);
numeric_into_atom!(u32);
numeric_into_atom!(u64);
numeric_into_atom!(u128);
numeric_into_atom!(usize);

numeric_into_atom!(i8);
numeric_into_atom!(i16);
numeric_into_atom!(i32);
numeric_into_atom!(i64);
numeric_into_atom!(i128);
numeric_into_atom!(isize);

numeric_into_atom!(f32);
numeric_into_atom!(f64);

#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub enum ExpressionData {
    Presence(Atom, LexicalSpan),
    Binding(String, Box<Expression>, LexicalSpan),
    List(Vec<Expression>, LexicalSpan)
}

impl ExpressionData {
    pub fn span(&self) -> LexicalSpan {
        match self {
            Presence(_, s) => *s,
            Binding(_, _, s) => *s,
            List(_, s) => *s
        }
    }
}

#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub struct Expression {
    pub data: ExpressionData,
    pub comment: Option<String>
}

fn indent(string: impl AsRef<str>, with:  impl AsRef<str>) -> String {
    format!("{}{}", with.as_ref(), string.as_ref().replace("\n", &format!("\n{}", with.as_ref())))
}

pub fn escape(message: impl AsRef<str>) -> String {
    message
        .as_ref()
        .replace("(", r"\(")
        .replace(")", r"\)")
        .replace("=", r"\=")
        .replace("#", r"\#")
}

impl Expression {
    pub fn new(data: ExpressionData, comment: Option<String>) -> Self {
        Self {
            data,
            comment,
        }
    }

    pub fn uncommented(data: ExpressionData) -> Expression {
        Self::new(data, None)
    }

    pub fn presence(value: impl Into<Atom>, lexical_span: LexicalSpan) -> Self {
        Self::new(Presence(value.into(), lexical_span), None)
    }
    pub fn binding(key: String, value: Expression, lexical_span: LexicalSpan) -> Self {
        Self::new(Binding(key, Box::new(value), lexical_span), None)
    }
    pub fn list(vec: Vec<Expression>, lexical_span: LexicalSpan) -> Self {
        Self::new(
            List(vec, lexical_span),
            None,
        )
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn uncomented_pretty(&self) -> String {
        match &self.data {
            Presence(s, _) => s.to_string(),
            Binding(s, e, _) => format!("{} = {}", s, e.pretty()),
            List(c, _) => format!(
                "(\n{}\n)",
                indent(c.iter().map(|x| x.pretty()).collect::<Vec<String>>().join("\n"), "\t")
            )
        }
    }
    pub fn pretty(&self) -> String {
        if let Some(comment) = &self.comment {
            format!("{}\n{}", indent(comment, "# "), self.uncomented_pretty())
        } else {
            self.uncomented_pretty()
        }
    }

    pub fn uncomented_dump(&self) -> String {
        match &self.data {
            Presence(s, _) => s.to_string(),
            Binding(s, e, _) => format!("{} = {}", s, e.dump()),
            List(c, _) => format!("({})", c.iter().map(|x| x.dump()).collect::<Vec<String>>().join(" "))
        }
    }
    pub fn dump(&self) -> String {
        if let Some(comment) = &self.comment {
            format!("{}\n{}", indent(comment, "# "), self.uncomented_dump())
        } else {
            self.uncomented_dump()
        }
    }

    /// Convert this Expression to the smallest form which holds it. I.E. `Collection`s with 1 element
    /// are replaced with that element, removing the collection.
    pub fn minimized(mut self) -> Self {
        if let List(ref mut c, _) = &mut self.data {
            if c.len() != 1 {
                return self;
            }

            return c.remove(0)
        }

        self
    }

    /// Get a value by key if it exists.
    pub fn get(&self, key: impl AsRef<str>) -> Option<Expression> {
        let key = key.as_ref();
        match &self.data {
            Presence(_, _) => Some(self.clone()),
            Binding(k, v, _) => {
                if key == k {
                    Some(*v.clone())
                } else {
                    None
                }
            }
            List(c, _) => {
                for e in c {
                    if let Some(e) = e.get(key) {
                        return Some(e);
                    }
                }

                None
            }
        }
    }

    /// Return the value stored in an `Expression` as a `&String` IFF it is a singular value.
    pub fn release(&self) -> Option<&Atom> {
        match &self.data {
            Presence(p, _) => Some(p),
            Binding(_, v, _) => v.release(),
            List(_, _) => None
        }
    }

    /// Check if two data blocks are the same, ignoring the positional information.
    pub fn data_equivalent(&self, other: &Self) -> bool {
        self.data == other.data
    }

    /// inserts a value into a list iff the value is a list.
    pub fn prepend_into_list(&mut self, expression: Expression) {
        match &mut self.data {
            List(v, _) => {
                v.insert(0, expression);
            }
            _ => {}
        }
    }

    pub fn span(&self) -> LexicalSpan {
        self.data.span()
    }
}

impl From<ExpressionData> for Expression {
    fn from(value: ExpressionData) -> Self {
        Self::new(value, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested() -> Expression {
        Expression::binding(String::from("alphabet"), Expression::list(vec![
            Expression::presence(String::from("a"), LexicalSpan::new(0, 0)),
            Expression::presence(String::from("b"), LexicalSpan::new(0, 0)),
            Expression::presence(String::from("c"), LexicalSpan::new(0, 0)),
        ], LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0))
    }

    fn more_nested() -> Expression {
        Expression::binding(String::from("alphabet"), Expression::list(vec![
            nested(),
            nested(),
        ], LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0))
    }
    #[test]
    fn dump() {
        assert_eq!(nested().dump(), String::from("alphabet = (a b c)"))
    }

    #[test]
    fn pretty() {
        assert_eq!(nested().pretty(), String::from("alphabet = (\n\ta\n\tb\n\tc\n)"))
    }

    #[test]
    fn pretty_more_nested() {
        assert_eq!(more_nested().pretty(), String::from("alphabet = (\n\talphabet = (\n\t\ta\n\t\tb\n\t\tc\n\t)\n\talphabet = (\n\t\ta\n\t\tb\n\t\tc\n\t)\n)"))
    }

    fn get_demo_expr() -> Expression {
        Expression::list(vec![
            Expression::binding("key".to_string(), Expression::presence("cat".to_string(), LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0)),
            Expression::binding("vec".to_string(), Expression::list(vec![
                Expression::presence("bird".to_string(), LexicalSpan::new(0, 0)),
                Expression::presence("dog".to_string(), LexicalSpan::new(0, 0)),
            ], LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0)),
        ], LexicalSpan::new(0, 0))
    }
    #[test]
    fn get() {
        let gotten = get_demo_expr().get("vec").unwrap();
        assert_eq!(gotten, Expression::list(vec![
            Expression::presence("bird".to_string(), LexicalSpan::new(0, 0)),
            Expression::presence("dog".to_string(), LexicalSpan::new(0, 0)),
        ], LexicalSpan::new(0, 0)))
    }

    #[test]
    fn pretty_harder() {
        let make_pretty = Expression::list(vec![
            Expression::binding("run_name".to_string(), Expression::presence("first".to_string(), LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0)),
            Expression::binding("names_in_run".to_string(), Expression::list(vec![
                Expression::list(vec![Expression::presence("HandMade".to_string(), LexicalSpan::new(0, 0)), Expression::presence("Ethan".to_string(), LexicalSpan::new(0, 0))], LexicalSpan::new(0, 0)),
                Expression::list(vec![Expression::presence("HandMade".to_string(), LexicalSpan::new(0, 0)), Expression::presence("James".to_string(), LexicalSpan::new(0, 0))], LexicalSpan::new(0, 0)),
                Expression::list(vec![Expression::presence("Generated".to_string(), LexicalSpan::new(0, 0)), Expression::presence("SDKJLHF".to_string(), LexicalSpan::new(0, 0))], LexicalSpan::new(0, 0)),
                Expression::list(vec![Expression::presence("Generated".to_string(), LexicalSpan::new(0, 0)), Expression::presence("Kerflooble".to_string(), LexicalSpan::new(0, 0))], LexicalSpan::new(0, 0)),
            ], LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0)),
            Expression::binding("count".to_string(), Expression::presence("2".to_string(), LexicalSpan::new(0, 0)), LexicalSpan::new(0, 0))
        ], LexicalSpan::new(0, 0));

        assert_eq!(make_pretty.pretty(),
"(\n\trun_name = first
\tnames_in_run = (
\t\t(
\t\t\tHandMade
\t\t\tEthan
\t\t)
\t\t(
\t\t\tHandMade
\t\t\tJames
\t\t)
\t\t(
\t\t\tGenerated
\t\t\tSDKJLHF
\t\t)
\t\t(
\t\t\tGenerated
\t\t\tKerflooble
\t\t)
\t)
\tcount = 2
)")
    }
}