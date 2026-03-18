use easy_config::EasyConfig;

#[derive(EasyConfig)]
enum Wrapper {
    Int(i32),
    Text(String),
    Float(f64),
}

fn main() {}
