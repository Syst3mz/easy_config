use easy_config::EasyConfig;

#[derive(EasyConfig)]
union Foo {
    a: u8,
    b: i8,
}

fn main() {}
