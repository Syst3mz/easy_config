use std::fmt::Display;
use crate::expression::ExpressionData::{List, BindingExpr, Presence};
use crate::expression_iterator::ExpressionIterator;
use crate::lexical_span::LexicalSpan;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Atom {
    Text(String),
    Number(String),
}

impl Atom {
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }
    pub fn is_number(&self) -> bool {matches!(self, Self::Number(_))}
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
pub struct Binding {
    pub name :String,
    pub value: Box<Expression>,
    pub span: LexicalSpan
}
#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub enum ExpressionData {
    Presence(Atom, LexicalSpan),
    BindingExpr(Binding),
    List(Vec<Expression>, LexicalSpan)
}

impl ExpressionData {
    pub fn span(&self) -> LexicalSpan {
        match self {
            Presence(_, s) => *s,
            BindingExpr(b) => b.span,
            List(_, s) => *s
        }
    }

    pub fn is_presence(&self) -> bool {
        matches!(self, Presence(_, _))
    }
    pub fn is_binding(&self) -> bool {
        matches!(self, BindingExpr(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, List(_, _))
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

    pub fn presence(value: impl Into<Atom>) -> Self {
        Self::new(Presence(value.into(), LexicalSpan::zeros()), None)
    }
    pub fn binding(key: impl AsRef<str>, value: Expression) -> Self {
        Self::new(BindingExpr(Binding {
            name: key.as_ref().to_string(),
            value: Box::new(value),             
            span: LexicalSpan::zeros(),
        }), None)
    }
    pub fn list(vec: Vec<Expression>) -> Self {
        Self::new(
            List(vec, LexicalSpan::zeros()),
            None,
        )
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
    
    pub fn with_span(mut self, lexical_span: LexicalSpan) -> Self {
        match &mut self.data {
            Presence(_, s)   |
            List(_, s)    => {*s = lexical_span},
            BindingExpr(b) => {b.span = lexical_span;}
        }
        
        self
    }

    pub fn uncomented_pretty(&self) -> String {
        match &self.data {
            Presence(s, _) => s.to_string(),
            BindingExpr(b) => format!("{} = {}", b.name, b.value.pretty()),
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
            BindingExpr(b) => format!("{} = {}", b.name, b.value.dump()),
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

    pub fn span(&self) -> LexicalSpan {
        self.data.span()
    }
    pub fn is_presence(&self) -> bool {
        self.data.is_presence()
    }
    pub fn is_binding(&self) -> bool {
        self.data.is_binding()
    }
    pub fn is_list(&self) -> bool {
        self.data.is_list()
    }

    // Prepend an expr into the list if this is a list.
    // Panics if self is not a list.
    pub fn prepend_into_list(&mut self, expr: Expression) {
        match &mut self.data {
            List(l, _) => l.insert(0, expr),
            _ => panic!("Attempted to prepend into something that was not a list!")
        }
    }

    #[cfg(debug_assertions)]
    pub fn get_first_expr_or_panic(self) -> Expression {
        match self.data {
            List(mut l, _) => l.remove(0),
            _ => panic!("Attempted to get the first value of a non-list expression.")
        }
    }
}

impl From<ExpressionData> for Expression {
    fn from(value: ExpressionData) -> Self {
        Self::new(value, None)
    }
}
impl IntoIterator for Expression {
    type Item = Expression;
    type IntoIter = ExpressionIterator;
    fn into_iter(self) -> Self::IntoIter {
        ExpressionIterator::new(self)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn nested() -> Expression {
        Expression::binding(String::from("alphabet"), Expression::list(vec![
            Expression::presence(String::from("a")),
            Expression::presence(String::from("b")),
            Expression::presence(String::from("c")),
        ]))
    }

    fn more_nested() -> Expression {
        Expression::binding(String::from("alphabet"), Expression::list(vec![
            nested(),
            nested(),
        ]))
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

    #[test]
    fn pretty_harder() {
        let make_pretty = Expression::list(vec![
            Expression::binding("run_name".to_string(), Expression::presence("first".to_string())),
            Expression::binding("names_in_run".to_string(), Expression::list(vec![
                Expression::list(vec![Expression::presence("HandMade".to_string()), Expression::presence("Ethan".to_string())]),
                Expression::list(vec![Expression::presence("HandMade".to_string()), Expression::presence("James".to_string())]),
                Expression::list(vec![Expression::presence("Generated".to_string()), Expression::presence("SDKJLHF".to_string())]),
                Expression::list(vec![Expression::presence("Generated".to_string()), Expression::presence("Kerflooble".to_string())]),
            ])),
            Expression::binding("count".to_string(), Expression::presence("2".to_string()))
        ]);

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