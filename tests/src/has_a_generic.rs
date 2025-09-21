use easy_config::serialization::EasyConfig as Ezc;
use easy_config_derive::EasyConfig;

#[derive(Debug, EasyConfig)]
pub struct JustAGeneric<T: Ezc> {
    x: T
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {

    }
}