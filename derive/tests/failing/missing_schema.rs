use easy_config::EasyConfig;

struct NoSchema;

#[derive(EasyConfig)]
struct Foo {
    a: u8,
    b: NoSchema,
}

fn main() {}
