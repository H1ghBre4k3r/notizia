/// Compile-fail tests for the #[Task] macro
/// These tests verify that the macro correctly rejects invalid code
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
