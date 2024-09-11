use std::fmt::{Display, Formatter};
use std::str::FromStr;
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

    /// Insert the value into the expression. Values which are not `Collection`s will be promoted to
    /// `Collection`s to support this.
    pub fn insert(&mut self, item: Expression) {
        match self {
            Presence(_) | Pair(_, _) => {
                let old = std::mem::replace(self, Presence(String::new()));
                *self = Collection(vec![old, item])
            }
            Collection(c) => c.push(item)
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

    /// Checks if there is a `Presence` which matches or a `Pair` whose key matches or if a
    /// `Collection`'s direct descendant matches.
    pub fn exists(&self, needle: impl AsRef<str>) -> bool {
        let needle = needle.as_ref();
        match self {
            Presence(s) => s == needle,
            Pair(s, _) => s == needle,
            Collection(collection) => {
                for expr in collection {
                    match expr {
                        Presence(_) | Pair(_, _) => return expr.exists(needle),
                        _ => { }
                    }
                }

                false
            }
        }
    }

    /// Checks if there is a `Presence` which matches or a `Pair` whose key matches or if that is
    /// true in any child expression. Returns a readonly reference to the matching expression if one
    /// is found.
    pub fn explore_for(&self, needle: impl AsRef<str>) -> Option<&Expression> {
        let needle = needle.as_ref();
        match self {
            Presence(s) => (s == needle).then(|| self),
            Pair(s, e) => {
                if s == needle {
                    return Some(self)
                }

                e.explore_for(needle)
            },
            Collection(collection) => {
                for expr in collection {
                    let explored = expr.explore_for(needle);
                    if explored.is_some() {
                        return explored;
                    }
                }

                None
            }
        }
    }

    /// Gets the string stored in a `Presence` if `self` is a `Presence`.
    pub fn get_presence_value(&self) -> Option<&String> {
        match self {
            Presence(s) => Some(s),
            _ => None
        }
    }


    /// Get the value of a key in an `Expression` if that key exists and has a single `Presence`
    /// value and the string stored in that `Presence` can be converted by `T::FromStr`.
    pub fn get<T: FromStr>(&self, needle: impl AsRef<str>) -> Option<Result<T, <T as FromStr>::Err>> {
        let needle = needle.as_ref();
        let maybe_pair = self.explore_for(needle)?;
        let Pair(_, value) = maybe_pair else { return None; };
        let value = value.get_presence_value()?;
        Some(T::from_str(value))
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
    use crate::parser::Parser;
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

    #[test]
    fn insert_single() {
        let mut start = Parser::new("a = 1").parse().unwrap();
        let addition = Parser::new("b = (1 2)").parse().unwrap();
        let correct = Parser::new("a = 1 b = (1 2)").parse().unwrap();
        start.insert(addition);
        assert_eq!(start, correct)
    }

    #[test]
    fn insert_collection() {
        let mut start = Parser::new("(1 2)").parse().unwrap();
        let addition = Parser::new("3").parse().unwrap();
        let correct = Parser::new("1 2 3").parse().unwrap();
        start.insert(addition);
        assert_eq!(start, correct)
    }

    #[test]
    fn exists() {
        let expr = Parser::new("a = 1 b = (1 2)").parse().unwrap();
        assert!(expr.exists("a"))
    }

    #[test]
    fn not_exists() {
        let expr = Parser::new("a = 1 b = (1 2 c = 2)").parse().unwrap();
        assert!(!expr.exists("z"))
    }

    #[test]
    fn not_exists_nesting() {
        let expr = Parser::new("a = 1 b = (1 2 c = 2)").parse().unwrap();
        assert!(!expr.exists("c"))
    }
}