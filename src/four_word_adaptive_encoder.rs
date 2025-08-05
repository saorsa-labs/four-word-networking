//! Adaptive four-word encoder that handles both IPv4 and IPv6 addresses.
//!
//! This is the main public API for four-word networking.

use crate::error::{FourWordError, Result};
use crate::four_word_encoder::FourWordEncoder;
use crate::four_word_ipv6_encoder::{FourWordIpv6Encoder, Ipv6FourWordGroupEncoding};
use std::net::{IpAddr, SocketAddr};

/// The main four-word networking encoder interface
pub struct FourWordAdaptiveEncoder {
    ipv4_encoder: FourWordEncoder,
    ipv6_encoder: FourWordIpv6Encoder,
}

impl FourWordAdaptiveEncoder {
    /// Creates a new four-word adaptive encoder
    pub fn new() -> Result<Self> {
        Ok(FourWordAdaptiveEncoder {
            ipv4_encoder: FourWordEncoder::new(),
            ipv6_encoder: FourWordIpv6Encoder::new(),
        })
    }

    /// Encodes any IP address into words
    /// - IPv4: Always exactly 4 words
    /// - IPv6: 6, 9, or 12 words based on compression
    pub fn encode(&self, input: &str) -> Result<String> {
        let addr = self.parse_address(input)?;

        match addr {
            SocketAddr::V4(_) => {
                let encoded = self.ipv4_encoder.encode(addr)?;
                Ok(encoded.to_string())
            }
            SocketAddr::V6(v6) => {
                let encoded = self.ipv6_encoder.encode(&v6)?;
                Ok(encoded.to_string())
            }
        }
    }

    /// Decodes words back to an IP address
    /// Port 65535 is treated as "no port specified" and omitted from output
    pub fn decode(&self, words: &str) -> Result<String> {
        // Determine separator and count words appropriately
        let word_count = if words.contains(' ') {
            // For space-separated words, filter out empty strings from trailing spaces
            words.split(' ').filter(|s| !s.is_empty()).count()
        } else if words.contains('.') {
            // Dot-separated (IPv4 or backwards compatibility)
            words.split('.').filter(|s| !s.is_empty()).count()
        } else if words.contains('-') {
            // Dash-separated (IPv6)
            words.split('-').filter(|s| !s.is_empty()).count()
        } else {
            // Single word or other format
            1
        };

        match word_count {
            4 => {
                // IPv4
                let addr = self.ipv4_encoder.decode(words)?;
                // If port is 65535, omit it (special marker for "no port specified")
                if addr.port() == 65535 {
                    Ok(addr.ip().to_string())
                } else {
                    Ok(addr.to_string())
                }
            }
            6 | 9 | 12 => {
                // IPv6 (6, 9, or 12 words including padding)
                let groups = self.parse_ipv6_groups(words)?;
                let decoded = self.ipv6_encoder.decode(&groups)?;
                let socket_addr = SocketAddr::V6(decoded);
                // If port is 65535, omit it (special marker for "no port specified")
                if socket_addr.port() == 65535 {
                    Ok(socket_addr.ip().to_string())
                } else {
                    Ok(socket_addr.to_string())
                }
            }
            _ => Err(FourWordError::InvalidWordCount {
                expected: 4, // or 6/8/12 for IPv6
                actual: word_count,
            }),
        }
    }

    /// Returns information about the encoding
    pub fn analyze(&self, input: &str) -> Result<String> {
        let addr = self.parse_address(input)?;

        match addr {
            SocketAddr::V4(v4) => Ok(format!(
                "IPv4 Address: {v4}\nEncoding: 4 words\nMethod: Perfect reconstruction"
            )),
            SocketAddr::V6(v6) => {
                let encoded = self.ipv6_encoder.encode(&v6)?;
                Ok(format!(
                    "IPv6 Address: {v6}\nCategory: {:?}\nEncoding: {} words\nMethod: Category-based compression",
                    encoded.category(),
                    encoded.word_count()
                ))
            }
        }
    }

    /// Parses an IP address string into a SocketAddr
    /// Uses port 65535 as a special marker for "no port specified"
    fn parse_address(&self, input: &str) -> Result<SocketAddr> {
        // Try parsing as socket address first
        if let Ok(addr) = input.parse::<SocketAddr>() {
            return Ok(addr);
        }

        // Try parsing as IP address (use port 65535 as marker for "no port specified")
        if let Ok(ip) = input.parse::<IpAddr>() {
            return Ok(match ip {
                IpAddr::V4(v4) => SocketAddr::new(IpAddr::V4(v4), 65535),
                IpAddr::V6(v6) => SocketAddr::new(IpAddr::V6(v6), 65535),
            });
        }

        Err(FourWordError::InvalidInput(format!(
            "Invalid IP address format: {input}"
        )))
    }

    /// Parses IPv6 word groups from a string
    fn parse_ipv6_groups(&self, words: &str) -> Result<Ipv6FourWordGroupEncoding> {
        use crate::four_word_ipv6_encoder::FourWordGroup;
        use crate::ipv6_compression::Ipv6Category;

        // Parse words and filter out empty strings
        let all_words: Vec<String> = if words.contains(' ') {
            words
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else if words.contains('.') {
            words
                .split('.')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else {
            words
                .split('-')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        // IPv6 can have 6, 9, or 12 words
        if all_words.len() != 6 && all_words.len() != 9 && all_words.len() != 12 {
            return Err(FourWordError::InvalidWordCount {
                expected: 6, // or 9/12
                actual: all_words.len(),
            });
        }

        let mut groups = Vec::new();

        // Create groups of 4 words, handling 6 and 9 word cases properly
        match all_words.len() {
            6 => {
                // For 6 words, create 1 group of 4 and keep the remaining 2 words separately
                // This is handled by creating only 1.5 groups but since we can't have half groups,
                // we'll encode the extra 2 words by creating a partial second group
                groups.push(FourWordGroup::new(
                    all_words[0].clone(),
                    all_words[1].clone(),
                    all_words[2].clone(),
                    all_words[3].clone(),
                ));
                // For the remaining 2 words, we need to store them in the compressed data
                // rather than creating empty padding. The IPv6 encoder will handle this.
                // We'll use a special marker approach in the FourWordGroup structure
                groups.push(FourWordGroup::new(
                    all_words[4].clone(),
                    all_words[5].clone(),
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                ));
            }
            9 => {
                // For 9 words, create 2 groups of 4 and keep the remaining 1 word separately
                groups.push(FourWordGroup::new(
                    all_words[0].clone(),
                    all_words[1].clone(),
                    all_words[2].clone(),
                    all_words[3].clone(),
                ));
                groups.push(FourWordGroup::new(
                    all_words[4].clone(),
                    all_words[5].clone(),
                    all_words[6].clone(),
                    all_words[7].clone(),
                ));
                // For the remaining 1 word, create a partial third group
                groups.push(FourWordGroup::new(
                    all_words[8].clone(),
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                ));
            }
            12 => {
                // For 12 words, create groups of 4
                for chunk in all_words.chunks(4) {
                    groups.push(FourWordGroup::new(
                        chunk.first().cloned().unwrap_or_default(),
                        chunk.get(1).cloned().unwrap_or_default(),
                        chunk.get(2).cloned().unwrap_or_default(),
                        chunk.get(3).cloned().unwrap_or_default(),
                    ));
                }
            }
            _ => {
                return Err(FourWordError::InvalidWordCount {
                    expected: 6,
                    actual: all_words.len(),
                })
            }
        };

        // For decoding, we don't know the category yet, so use a placeholder
        // The actual category will be extracted during decoding from the encoded data
        Ok(Ipv6FourWordGroupEncoding::new(
            groups,
            Ipv6Category::GlobalUnicast, // placeholder - will be replaced during decoding
        ))
    }
}

impl Default for FourWordAdaptiveEncoder {
    fn default() -> Self {
        Self::new().expect("Failed to create encoder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_encoding() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        let test_cases = vec![
            "192.168.1.1:443",
            "10.0.0.1:80",
            "127.0.0.1:8080",
            "172.16.0.1:22",
        ];

        for addr in test_cases {
            let encoded = encoder.encode(addr).unwrap();
            let word_count = encoded.split_whitespace().count();
            assert_eq!(word_count, 4);

            let decoded = encoder.decode(&encoded).unwrap();
            assert_eq!(addr, decoded);
        }
    }

    #[test]
    fn test_ipv6_encoding() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

        for addr in test_cases {
            let encoded = encoder.encode(addr).unwrap();
            let word_count = encoded.split_whitespace().count();
            assert!(word_count == 6 || word_count == 9 || word_count == 12);
            assert!(word_count == 6 || word_count == 9 || word_count == 12); // 6, 9, or 12 words

            let decoded = encoder.decode(&encoded).unwrap();
            // Compare just the IP and port (not the full format)
            let original: SocketAddr = addr.parse().unwrap();
            let decoded_addr: SocketAddr = decoded.parse().unwrap();
            assert_eq!(original.ip(), decoded_addr.ip());
            assert_eq!(original.port(), decoded_addr.port());
        }
    }

    #[test]
    fn test_analyze() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        let analysis = encoder.analyze("192.168.1.1:443").unwrap();
        assert!(analysis.contains("IPv4"));
        assert!(analysis.contains("4 words"));

        let analysis = encoder.analyze("[::1]:443").unwrap();
        assert!(analysis.contains("IPv6"));
        assert!(analysis.contains("words"));
    }
}
