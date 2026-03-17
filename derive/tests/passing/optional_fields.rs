use easy_config::EasyConfig;

#[derive(EasyConfig)]
struct Foo {
    required: u8,
    optional: Option<String>,
    nested_option: Option<Option<i32>>,
}

fn main() {}
