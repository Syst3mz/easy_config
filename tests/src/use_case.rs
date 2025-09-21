use easy_config_derive::EasyConfig;

#[derive(EasyConfig, PartialEq, Debug, Eq)]
struct ServerTest {
    address: String,
    port: u16,
    name: String,
}

#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
    use super::*;

    fn localhost() -> ServerTest {
        ServerTest {
            address: "localhost".to_string(),
            port: 1337,
            name: "h4x0r".to_string(),
        }
    }

    #[test]
    fn read_human_input() {
        let input = r"address = localhost
        port = 1337
        name = h4x0r";
        let parsed = Parser::new(input).parse().unwrap();
        assert_eq!(ServerTest::deserialize(&mut parsed.into_iter(), input).unwrap(), localhost());
    }
}