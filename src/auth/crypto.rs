//! Cryptography related functions.

use crypto::digest::Digest;
use crypto::sha3::Sha3;

/// Hash password
pub fn hash_password(password: &String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(password);
    hasher.result_str()
}
