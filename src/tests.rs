use crate::crypto::{sign_message, verify_signature};

const TEST_KEY: &[u8] = b"crMwNFYF1cPeFqC16h43viK87zSEqlvt";

// test create signature and verify
#[test]
fn test_sign_and_verify() {
    let signature = sign_message(TEST_KEY, b"world");
    assert_eq!(verify_signature(TEST_KEY, b"world", &signature), true);
}

#[test]
fn test_verify_bad_signature() {
    assert_eq!(verify_signature(TEST_KEY, b"world", "bad_signature"), false);
    let signature = sign_message(TEST_KEY, b"world");
    assert_eq!(verify_signature(TEST_KEY, b"world2", &signature), false);
}
