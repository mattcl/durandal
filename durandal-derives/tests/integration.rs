#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cli-dispatch.rs");
    t.pass("tests/cli-meta-dispatch.rs");
}
