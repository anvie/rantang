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
use std::{
    fs::File,
    io::{self, Read},
};

use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};

pub(crate) fn verify_signature(key: &[u8], message: &[u8], signature: &str) -> bool {
    let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(message);
    // let expected_signature = mac.finalize();

    let provided_signature = match hex::decode(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    mac.verify_slice(provided_signature.as_slice()).is_ok()
}

#[allow(dead_code)]
pub(crate) fn sign_message(key: &[u8], message: &[u8]) -> String {
    let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(message);
    let signature = mac.finalize();
    hex::encode(signature.into_bytes())
}

/// Function to get SHA1 hash of file
pub(crate) fn get_sha1_file(file: &mut File) -> Result<String, io::Error> {
    let mut sha1 = Sha1::new();
    let mut buf = [0; 1024];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha1.update(&buf[..n]);
    }

    Ok(hex::encode(sha1.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_hash() {
        let mut file = File::open("IMG_9211.jpg").unwrap();
        let hash = get_sha1_file(&mut file).unwrap();
        assert_eq!(hash, "e1586b201c06a2d440358378f15d6a7987ee4ab6");
    }
}
