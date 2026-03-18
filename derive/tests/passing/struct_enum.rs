use easy_config::EasyConfig;

#[derive(EasyConfig)]
enum Event {
    Moved { x: f32, y: f32 },
    Resized { width: u32, height: u32 },
    Closed,
}

fn main() {}
