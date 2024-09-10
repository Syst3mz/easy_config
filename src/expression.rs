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

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Expression::Presence(s) => s.to_string(),
            Expression::Pair(s, e) => format!("{} = {}", s, e.to_string()),
            Expression::Collection(s) => {
                if f.alternate() {
                    format!("(\n{}\n)", indent(s.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"), "\t"))
                } else {
                    format!("({})", s.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" "))
                }
            }
        })
    }
}