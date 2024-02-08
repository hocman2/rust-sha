//! Strict SHA-256 and SHA-512 implementation using this paper: https://csrc.nist.gov/files/pubs/fips/180-2/final/docs/fips180-2.pdf
//! Usage:
//! ```
//! use sha::two_five_six::hash;
//! 
//! fn main() {
//!     let hash_result: [u8; 32] = hash(b"Hello, World!");
//! }
//! ```

mod preprocessing;
mod hasher;
pub mod two_five_six;
pub mod five_twelve;