use easy_config_derive::EasyConfig;

#[allow(dead_code)]
type ItemId = String;

#[allow(dead_code)]
#[derive(Debug, Clone, EasyConfig, PartialEq)]
pub struct ItemDefinition {
    id: ItemId,
    name: String,
    tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
    use easy_config::serialization::EasyConfig;
    use super::*;

    fn expected() -> ItemDefinition {
        ItemDefinition {
            id: "WHITE_MANA".to_string(),
            name: "White Mana".to_string(),
            tags: vec![],
        }
    }

    #[test]
    fn deserialize_single() {
        let text = r"
            ItemDefinition (
                id = WHITE_MANA
                name = (White Mana)
                tags = ()
            )
        ";

        let parsed = Parser::new(text).parse().unwrap();
        let got = ItemDefinition::deserialize(&mut parsed.into_iter(), &text).unwrap();
        assert_eq!(got, expected())
    }

    #[test]
    fn deserialize_vec() {
        let text = r"
            ItemDefinition (
                id = WHITE_MANA
                name = (White Mana)
                tags = ()
            )
        ";

        let parsed = Parser::new(text).parse().unwrap();
        let got = Vec::<ItemDefinition>::deserialize(&mut parsed.into_iter(), &text).unwrap();
        assert_eq!(got, vec![expected()])
    }
}