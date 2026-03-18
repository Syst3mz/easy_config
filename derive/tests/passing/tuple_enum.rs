use easy_config::EasyConfig;

#[derive(EasyConfig)]
enum Shape {
    Circle(f32),
    Rectangle(f32, f32),
    Triangle(f32, f32, f32),
}

fn main() {}
