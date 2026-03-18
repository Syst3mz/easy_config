use easy_config::EasyConfig;

#[derive(EasyConfig)]
enum Mixed {
    Unit,
    NewType(u8),
    Tuple(u8, i32),
    Struct { a: u8, b: String },
}

fn main() {}
