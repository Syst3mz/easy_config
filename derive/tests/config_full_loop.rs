#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
use easy_config::parser::expression::Expression;
    use derive::Config;
    use easy_config::serialization::Config;
    use easy_config::serialization::DeserializeExtension;

    #[derive(PartialEq, Config, Debug)]
    struct Demo {
        key: String,
        vec: Vec<String>,
    }

    fn demo() -> Demo {
        Demo {
            key: "cat".to_string(),
            vec: vec!["bird".to_string(), "dog".to_string()],
        }
    }

    #[test]
    fn serialize() {
        let d = demo();
        assert_eq!(d.serialize().dump(), "(key = cat vec = (bird dog))")
    }

    #[test]
    fn deserialize() {
        let parsed = Parser::new(demo().serialize().dump()).parse().unwrap();
        assert_eq!(
            Demo::deserialize(parsed).unwrap(), demo()
        )
    }


    /*
    #[derive(PartialEq, Config, Debug)]
    struct Address {
        to_name: String,
        street_address: String,
        street_address_second_line: Option<String>,
        city: String,
        state: String,
        zip: u32
    }

    #[derive(PartialEq, Config, Debug)]
    struct HardDemo {
        order_name: String,
        home: Address,
    }

    fn harder_demo() {
        HardDemo {
            order_name: "Hot dogs".to_string(),
            home: Address {
                to_name: "Ethan".to_string(),
                street_address: "12345 E wherever place".to_string(),
                street_address_second_line: None,
                city: "somewhere".to_string(),
                state: "wherever".to_string(),
                zip: 99999,
            },
        };
    }

    */
}