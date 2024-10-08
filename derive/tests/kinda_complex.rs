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
    use easy_config::parser::Parser;
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

    #[should_panic]
    #[test]
    fn good_error_test() {
        let text = "
        (
            run_name = (Error Out)
	        names_in_run = (
	        	(
	        		HandMade
	        	)
	        	(
	        		HandMade
	        		James
	        	)
	        	(
	        		Generated
	        		SDKJLHF
	        	)
	        	(
	        		Generated
	        		Kerflooble
	        	)
	        )
	        count = 2
	    )
";
        let parsed = Parser::parse(text).unwrap();
        let err = Configuration::deserialize(parsed).unwrap_err();
        assert_eq!(err.to_string(), "Expected a `alloc::string::String` in a `Configuration` did not find it in: (\n\tHandMade\n). @ (5, 11)")
    }
}