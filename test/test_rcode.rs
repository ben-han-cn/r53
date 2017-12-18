use r53::*;


#[test]
pub fn test_rcode_equal() {
    assert!(Rcode::no_error().is_no_error())
}
