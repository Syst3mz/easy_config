use easy_config::EasyConfig;

#[derive(EasyConfig)]
enum Color {
    Rgb(u8, u8, u8),
    Named(String),
}

#[derive(EasyConfig)]
struct Theme {
    primary: Color,
    secondary: Color,
    font_size: u32,
}

fn main() {}
