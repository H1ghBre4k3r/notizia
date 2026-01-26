/// Macro expansion snapshot tests using macrotest
/// These tests verify that the #[Task] macro expands to the correct code
#[test]
fn expansion_tests() {
    macrotest::expand("tests/expand/*.rs");
}
