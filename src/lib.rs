//! Four-Word Networking
//!
//! Convert network IP addresses into memorable word combinations
//! for human-friendly networking.
//!
//! ## Features
//!
//! - **Perfect IPv4**: Converts IPv4 addresses like `192.168.1.1:443`
//!   into exactly 4 memorable words with 100% perfect reconstruction
//! - **Adaptive IPv6**: Converts IPv6 addresses into 6, 9, or 12 words using intelligent compression
//! - **Voice-Friendly**: Easy to share over phone calls or voice chat
//! - **Error-Resistant**: Much less prone to typos than long technical addresses
//! - **Deterministic**: Same IP address always produces the same word combination
//! - **Visual Distinction**: Different formatting for IPv4 vs IPv6 addresses
//! - **Universal**: Works with any valid IP address format
//!
//! ## Examples
//!
//! ### Basic Encoding/Decoding
//! ```rust
//! use four_word_networking::FourWordAdaptiveEncoder;
//!
//! let encoder = FourWordAdaptiveEncoder::new()?;
//! let address = "192.168.1.1:443";
//!
//! // Convert to four words (perfect reconstruction for IPv4)
//! let words = encoder.encode(address)?;
//! println!("Address: {} -> {}", address, words);
//!
//! // Decode back to exact address
//! let decoded = encoder.decode(&words)?;
//! assert_eq!(address, decoded);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Random Word Generation
//! ```rust
//! use four_word_networking::FourWordAdaptiveEncoder;
//!
//! let encoder = FourWordAdaptiveEncoder::new()?;
//!
//! // Generate random dictionary words (NOT IP encodings)
//! // Useful for passphrases, test data, or any application needing random words
//! let random_words = encoder.get_random_words(4);
//! println!("Random words: {}", random_words.join(" "));
//!
//! // Generate a 6-word passphrase
//! let passphrase = encoder.get_random_words(6);
//! println!("Passphrase: {}", passphrase.join("-"));
//!
//! // All generated words are valid dictionary words
//! for word in &random_words {
//!     assert!(encoder.is_valid_word(word));
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Word Validation
//! ```rust
//! use four_word_networking::FourWordAdaptiveEncoder;
//!
//! let encoder = FourWordAdaptiveEncoder::new()?;
//!
//! // Validate user input words
//! assert!(encoder.is_valid_word("ocean"));
//! assert!(encoder.is_valid_word("OCEAN")); // Case-insensitive
//! assert!(!encoder.is_valid_word("xyz123")); // Invalid word
//!
//! // Get word suggestions for partial input
//! let hints = encoder.get_word_hints("oce");
//! assert!(hints.contains(&"ocean".to_string()));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod compression;
pub mod dictionary4k;
pub mod error;
pub mod four_word_adaptive_encoder;
pub mod four_word_encoder;
pub mod four_word_ipv6_encoder;
// Experimental modules removed
pub mod ipv6_compression;
pub mod ipv6_pattern_feistel;
pub mod ipv6_perfect_patterns;
pub mod pure_ip_compression;
// Ultra modules removed - used outdated 3-word system
pub mod universal_ip_compression;
pub mod validation;
pub mod identity_encoder;

#[cfg(test)]
mod property_tests;

pub use error::{FourWordError, Result};
// Main API - Four-word encoding
pub use four_word_adaptive_encoder::FourWordAdaptiveEncoder;
pub use four_word_encoder::{FourWordEncoder, FourWordEncoding};
pub use four_word_ipv6_encoder::{FourWordGroup, FourWordIpv6Encoder, Ipv6FourWordGroupEncoding};
// Compression and IPv6 support modules
pub use ipv6_compression::{CompressedIpv6, Ipv6Category, Ipv6Compressor};
pub use ipv6_pattern_feistel::{IPv6PatternFeistel, IPv6PatternId};
pub use ipv6_perfect_patterns::{IPv6Pattern, IPv6PatternDetector};
pub use pure_ip_compression::{MathematicalCompressor, PureIpCompressor};
pub use universal_ip_compression::UniversalIpCompressor;
pub use identity_encoder::{IdentityEncoder, IdentityWords};

/// Version of the four-word networking library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_four_word_functionality() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();
        let address = "192.168.1.1:443";

        let words = encoder.encode(address).unwrap();
        let word_count = words.split_whitespace().count();
        assert_eq!(word_count, 4);

        let decoded = encoder.decode(&words).unwrap();
        assert_eq!(address, decoded);
    }
}
