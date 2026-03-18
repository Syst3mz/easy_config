use easy_config::EasyConfig;

struct NoSchema;

#[derive(EasyConfig)]
enum Foo {
    Bar(NoSchema),
}

fn main() {}
