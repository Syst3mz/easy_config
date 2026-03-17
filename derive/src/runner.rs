#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/passing/*.rs");
    t.compile_fail("tests/failing/*.rs");
}