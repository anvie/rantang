/// MIT License
///
/// Copyright (c) 2023 Robin Syihab <r@nu.id>
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"),
/// to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
/// and/or sell copies of the Software and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
/// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
/// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF
/// OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
///
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
