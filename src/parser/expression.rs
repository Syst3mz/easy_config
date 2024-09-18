use std::fmt::{Display, Formatter};
use crate::parser::expression::Expression::{Collection, Pair, Presence};

#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub enum Expression {
    Presence(String),
    Pair(String, Box<Expression>),
    Collection(Vec<Expression>)
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
    pub fn escaped_pretty(&self) -> String {
        escape(self.pretty())
    }
    pub fn pretty(&self) -> String {
        match self {
            Presence(s) => s.clone(),
            Pair(s, e) => format!("{} = {}", s, e.pretty()),
            Collection(c) => format!("(\n{}\n)",
                                                 indent(c.iter().map(|x| x.pretty()).collect::<Vec<String>>().join("\n"), "\t")
            )
        }
    }

    pub fn escaped_dump(&self) -> String {
        escape(self.dump())
    }
    pub fn dump(&self) -> String {
        match self {
            Presence(s) => s.clone(),
            Pair(s, e) => format!("{} = {}", s, e.dump()),
            Collection(c) => format!("({})", c.iter().map(|x| x.dump()).collect::<Vec<String>>().join(" "))
        }
    }

    /// Convert this Expression to the smallest form which holds it. I.E. `Collection`s with 1 element
    /// are replaced with that element, removing the collection.
    pub fn minimized(mut self) -> Self{
        if let Collection(ref mut c) = &mut self {
            if c.len() != 1 {
                return self;
            }

            return c.remove(0)
        }

        self
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<Expression> {
        let key = key.as_ref();
        match self {
            Presence(p) => Some(Presence(p.to_string())),
            Pair(k, v) => {
                if key == k {
                    Some(*v.clone())
                } else {
                    None
                }
            }
            Collection(c) => {
                for e in c {
                    if let Some(e) = e.get(key) {
                        return Some(e);
                    }
                }

                None
            }
        }
    }
    pub fn release(&self) -> Option<&String> {
        match self {
            Presence(p) => Some(p),
            Pair(_, v) => v.release(),
            Collection(_) => None
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
        Pair(String::from("alphabet"), Box::new(Collection(vec![
            Presence(String::from("a")),
            Presence(String::from("b")),
            Presence(String::from("c")),
        ])))
    }

    fn more_nested() -> Expression {
        Pair(String::from("alphabet"), Box::new(Collection(vec![
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

    fn get_demo_expr() -> Expression {
        Collection(vec![
            Pair("key".to_string(), Box::new(Presence("cat".to_string()))),
            Pair("vec".to_string(), Box::new(Collection(vec![
                Presence("bird".to_string()),
                Presence("dog".to_string()),
            ]))),
        ])
    }
    #[test]
    fn get() {
        let gotten = get_demo_expr().get("vec").unwrap();
        assert_eq!(gotten, Collection(vec![
            Presence("bird".to_string()),
            Presence("dog".to_string()),
        ]))
    }
}