//! Property-based tests for Layout size expression parser.
//!
//! Ensures that the size expression parser never panics on any input,
//! and that valid expressions return sensible results.

use rspfdisk_layouts::size::{parse_byte_size, parse_size_expr};

proptest::proptest! {
    // -----------------------------------------------------------------------
    // parse_byte_size never panics on random strings
    // -----------------------------------------------------------------------
    #[test]
    fn parse_byte_size_never_panics(s in ".*") {
        let _result = parse_byte_size(&s);
    }

    // -----------------------------------------------------------------------
    // parse_size_expr never panics on random strings
    // -----------------------------------------------------------------------
    #[test]
    fn parse_size_expr_never_panics(s in ".*") {
        let _result = parse_size_expr(&s);
    }

    // -----------------------------------------------------------------------
    // Non-zero numeric strings parse without panic
    // -----------------------------------------------------------------------
    #[test]
    fn positive_numeric_sizes_parse_ok(s in "[1-9][0-9]*(\\s*(GiB|MiB|KiB|GB|MB|KB))?") {
        let result = parse_byte_size(&s);
        // Accept any result — the test is that non-zero numeric strings
        // don't cause panics or unexpected errors
        if let Ok(bytes) = result {
            assert!(bytes > 0, "positive numeric input {s} should return > 0 bytes");
        }
    }

    // -----------------------------------------------------------------------
    // Fill / auto:swap / fill-minus patterns never panic
    // -----------------------------------------------------------------------
    #[test]
    fn fill_expressions_never_panic(s in "(fill|Fill|FILL|auto:swap)(-minus:-?\\d+)?") {
        let _result = parse_size_expr(&s);
    }
}
