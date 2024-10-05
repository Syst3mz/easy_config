use easy_config_derive::Config;

#[derive(Debug, Config, PartialEq)]
pub enum Kind {
    Generated,
    HandMade,
}

#[derive(Debug, Config, PartialEq)]
pub struct Configuration {
    pub run_name: String,
    pub names_in_run: Vec<(Kind, String)>,
    pub count: usize
}

#[cfg(test)]
mod tests {
    use crate::Kind::{Generated, HandMade};
    use super::*;
    use easy_config::serialization::Config;

    fn demo() -> Configuration {
        Configuration {
            run_name: "first".to_string(),
            names_in_run: vec![
                (HandMade, "Ethan".to_string()),
                (HandMade, "James".to_string()),
                (Generated, "SDKJLHF".to_string()),
                (Generated, "Kerflooble".to_string()),
            ],
            count: 2,
        }
    }

    #[test]
    fn serialize() {
        assert_eq!(demo().serialize().dump(), "(run_name = first names_in_run = ((HandMade Ethan) (HandMade James) (Generated SDKJLHF) (Generated Kerflooble)) count = 2)")
    }
}