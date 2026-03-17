use easy_config::EasyConfig;

#[derive(EasyConfig)]
struct Inner {
    x: u8,
    y: u8,
}

#[derive(EasyConfig)]
struct Outer {
    inner: Inner,
    name: String,
}

fn main() {}
