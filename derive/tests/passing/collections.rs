use easy_config::EasyConfig;
use std::collections::{BTreeMap, HashMap};

#[derive(EasyConfig)]
struct Foo {
    items: Vec<u8>,
    lookup: HashMap<String, i32>,
    ordered: BTreeMap<String, i32>,
}

fn main() {}
