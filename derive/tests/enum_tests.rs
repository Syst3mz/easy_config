use derive::Config;

#[derive(Config)]
enum Mode {
    First,
    Second(f32),
}

#[test]
fn serialize_mode_one() {
    let x = Mode::First;
    assert_eq!(x.serialize(), "First = ()")
}