use easy_config::EasyConfig;

#[derive(EasyConfig)]
struct Foo {
    a: u8,
    b: i32,
    c: String,
}

fn main() {}