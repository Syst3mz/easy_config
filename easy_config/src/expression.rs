use crate::expression::CstData::{Collection, Pair, Presence};
use crate::location::Location;

#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub enum CstData {
    Presence(String),
    Pair(String, Box<CstExpression>),
    Collection(Vec<CstExpression>)
}

#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub struct CstExpression {
    pub data: CstData,
    pub location: Option<Location>,
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

impl CstExpression {
    pub fn new(data: CstData, location: Option<Location>, comment: Option<String>) -> Self {
        Self {
            data,
            location,
            comment,
        }
    }
    pub fn uncommented(data: CstData, location: Location) -> CstExpression {
        Self::new(data, Some(location), None)
    }
    pub fn unlocated(data: CstData, comment: String) -> Self {
        Self::new(data, None, Some(comment))
    }

    pub fn presence(value: String) -> Self {
        Self::new(CstData::Presence(value), None, None)
    }
    pub fn pair(key: String, value: CstExpression) -> Self {
        Self::new(CstData::Pair(key, Box::new(value)), None, None)
    }
    pub fn collection(vec: Vec<CstExpression>) -> Self {
        Self::new(
            CstData::Collection(vec),
            None,
            None
        )
    }

    pub fn uncomented_pretty(&self) -> String {
        match &self.data {
            Presence(s) => s.clone(),
            Pair(s, e) => format!("{} = {}", s, e.pretty()),
            Collection(c) => format!(
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
            Presence(s) => s.clone(),
            Pair(s, e) => format!("{} = {}", s, e.dump()),
            Collection(c) => format!("({})", c.iter().map(|x| x.dump()).collect::<Vec<String>>().join(" "))
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
        if let Collection(ref mut c) = &mut self.data {
            if c.len() != 1 {
                return self;
            }

            return c.remove(0)
        }

        self
    }

    /// Get a value by key if it exists.
    pub fn get(&self, key: impl AsRef<str>) -> Option<CstExpression> {
        let key = key.as_ref();
        match &self.data {
            Presence(_) => Some(self.clone()),
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

    /// Return the value stored in an `Expression` as a `&String` IFF it is a singular value.
    pub fn release(&self) -> Option<&String> {
        match &self.data {
            Presence(p) => Some(p),
            Pair(_, v) => v.release(),
            Collection(_) => None
        }
    }

    /// Check if two data blocks are the same, ignoring the positional information.
    pub fn data_equivalent(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl From<CstData> for CstExpression {
    fn from(value: CstData) -> Self {
        Self::new(value, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested() -> CstExpression {
        CstExpression::pair(String::from("alphabet"), CstExpression::collection(vec![
            CstExpression::presence(String::from("a")),
            CstExpression::presence(String::from("b")),
            CstExpression::presence(String::from("c")),
        ]))
    }

    fn more_nested() -> CstExpression {
        CstExpression::pair(String::from("alphabet"), CstExpression::collection(vec![
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

    fn get_demo_expr() -> CstExpression {
        CstExpression::collection(vec![
            CstExpression::pair("key".to_string(), CstExpression::presence("cat".to_string())),
            CstExpression::pair("vec".to_string(), CstExpression::collection(vec![
                CstExpression::presence("bird".to_string()),
                CstExpression::presence("dog".to_string()),
            ])),
        ])
    }
    #[test]
    fn get() {
        let gotten = get_demo_expr().get("vec").unwrap();
        assert_eq!(gotten, CstExpression::collection(vec![
            CstExpression::presence("bird".to_string()),
            CstExpression::presence("dog".to_string()),
        ]))
    }

    #[test]
    fn pretty_harder() {
        let make_pretty = CstExpression::collection(vec![
            CstExpression::pair("run_name".to_string(), CstExpression::presence("first".to_string())),
            CstExpression::pair("names_in_run".to_string(), CstExpression::collection(vec![
                CstExpression::collection(vec![CstExpression::presence("HandMade".to_string()), CstExpression::presence("Ethan".to_string())]),
                CstExpression::collection(vec![CstExpression::presence("HandMade".to_string()), CstExpression::presence("James".to_string())]),
                CstExpression::collection(vec![CstExpression::presence("Generated".to_string()), CstExpression::presence("SDKJLHF".to_string())]),
                CstExpression::collection(vec![CstExpression::presence("Generated".to_string()), CstExpression::presence("Kerflooble".to_string())]),
            ])),
            CstExpression::pair("count".to_string(), CstExpression::presence("2".to_string()))
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