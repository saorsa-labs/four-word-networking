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
//! ## Example
//!
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
