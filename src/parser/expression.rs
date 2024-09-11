use std::fmt::{Display, Formatter};

#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub enum Expression {
    Presence(String),
    Pair(String, Box<Expression>),
    Collection(Vec<Expression>)
}

fn indent(string: impl AsRef<str>, with:  impl AsRef<str>) -> String {
    format!("{}{}", with.as_ref(), string.as_ref().replace("\n", &format!("\n{}", with.as_ref())))
}

impl Expression {
    fn pretty(&self) -> String {
        match self {
            Expression::Presence(s) => s.clone(),
            Expression::Pair(s, e) => format!("{} = {}", s, e.pretty()),
            Expression::Collection(c) => format!("(\n{}\n)",
                                                 indent(c.iter().map(|x| x.pretty()).collect::<Vec<String>>().join("\n"), "\t")
            )
        }
    }

    fn dump(&self) -> String {
        match self {
            Expression::Presence(s) => s.clone(),
            Expression::Pair(s, e) => format!("{} = {}", s, e.dump()),
            Expression::Collection(c) => format!("({})", c.iter().map(|x| x.dump()).collect::<Vec<String>>().join(" "))
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",if f.alternate() { self.pretty() } else { self.dump() })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::expression::Expression::{Collection, Presence};
    use super::*;

    fn nested() -> Expression {
        Expression::Pair(String::from("alphabet"), Box::new(Collection(vec![
            Presence(String::from("a")),
            Presence(String::from("b")),
            Presence(String::from("c")),
        ])))
    }

    fn more_nested() -> Expression {
        Expression::Pair(String::from("alphabet"), Box::new(Collection(vec![
            nested(),
            nested(),
        ])))
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
}