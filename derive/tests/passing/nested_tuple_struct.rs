use easy_config::EasyConfig;

#[derive(EasyConfig)]
struct Point(f32, f32);

#[derive(EasyConfig)]
struct Line(Point, Point);

fn main() {}
