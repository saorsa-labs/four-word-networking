//! Four-word encoder for IPv6 addresses.
//!
//! This module provides encoding and decoding of IPv6 addresses
//! into groups of four words using a 4,096-word dictionary.

use crate::dictionary4k::DICTIONARY;
use crate::error::{FourWordError, Result};
use crate::ipv6_compression::{CompressedIpv6, Ipv6Category, Ipv6Compressor};
use std::net::SocketAddrV6;

/// Represents a group of four words
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FourWordGroup {
    words: [String; 4],
}

impl FourWordGroup {
    /// Creates a new four-word group
    pub fn new(w1: String, w2: String, w3: String, w4: String) -> Self {
        FourWordGroup {
            words: [w1, w2, w3, w4],
        }
    }

    /// Returns the words as an array
    pub fn words(&self) -> &[String; 4] {
        &self.words
    }
}

impl std::fmt::Display for FourWordGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.words.join(" "))
    }
}

/// IPv6 encoding using groups of four words
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6FourWordGroupEncoding {
    /// Groups of four words
    groups: Vec<FourWordGroup>,
    /// Original IPv6 category for reconstruction
    category: Ipv6Category,
}

impl Ipv6FourWordGroupEncoding {
    /// Creates a new IPv6 four-word group encoding
    pub fn new(groups: Vec<FourWordGroup>, category: Ipv6Category) -> Self {
        Ipv6FourWordGroupEncoding { groups, category }
    }

    /// Returns the word groups
    pub fn groups(&self) -> &[FourWordGroup] {
        &self.groups
    }

    /// Returns the total word count (excluding empty padding words)
    pub fn word_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|g| g.words().iter())
            .filter(|w| !w.is_empty())
            .count()
    }

    /// Returns the IPv6 category
    pub fn category(&self) -> Ipv6Category {
        self.category
    }

    /// Formats as dash-separated groups (visual distinction from IPv4)
    pub fn to_dashed_string(&self) -> String {
        self.groups
            .iter()
            .flat_map(|g| g.words().iter())
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

impl std::fmt::Display for Ipv6FourWordGroupEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all_words: Vec<&str> = self
            .groups
            .iter()
            .flat_map(|g| g.words().iter())
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .collect();
        write!(f, "{}", all_words.join(" "))
    }
}

/// Four-word encoder for IPv6 addresses
pub struct FourWordIpv6Encoder {
    compressor: Ipv6Compressor,
}

impl FourWordIpv6Encoder {
    /// Creates a new IPv6 four-word encoder
    pub fn new() -> Self {
        FourWordIpv6Encoder {
            compressor: Ipv6Compressor::new(),
        }
    }

    /// Encodes an IPv6 socket address into groups of four words
    pub fn encode(&self, addr: &SocketAddrV6) -> Result<Ipv6FourWordGroupEncoding> {
        // Compress the IPv6 address
        let compressed = self.compressor.compress(*addr.ip(), Some(addr.port()))?;
        let category = compressed.category;
        let compressed_data = compressed.as_bytes();

        // Each group of 4 words encodes 48 bits (4 * 12 bits)
        // We need to handle variable-length compressed data
        let groups = self.encode_bytes_to_groups(&compressed_data, addr.port(), category)?;

        Ok(Ipv6FourWordGroupEncoding::new(groups, category))
    }

    /// Decodes groups of four words back to an IPv6 socket address
    pub fn decode(&self, encoding: &Ipv6FourWordGroupEncoding) -> Result<SocketAddrV6> {
        // Decode groups back to bytes, port, and actual category
        let (decoded_bytes, decoded_port, actual_category) =
            self.decode_groups_to_bytes(&encoding.groups, encoding.category)?;

        // Create compressed IPv6 from bytes and actual category
        let mut compressed = CompressedIpv6::from_bytes(&decoded_bytes, actual_category)?;
        compressed.port = Some(decoded_port);

        // Decompress to get the original address
        let (addr, _) = self.compressor.decompress(&compressed)?;

        Ok(SocketAddrV6::new(addr, decoded_port, 0, 0))
    }

    /// Encodes bytes into groups of four words
    fn encode_bytes_to_groups(
        &self,
        data: &[u8],
        port: u16,
        category: Ipv6Category,
    ) -> Result<Vec<FourWordGroup>> {
        let mut groups = Vec::new();

        // Store the category (3 bits) + data length (5 bits) in the first byte, then data, then port
        // This way the decoder knows the category and exactly how many bytes to extract
        let data_len = data.len() as u8;
        if data_len > 31 {
            return Err(FourWordError::InvalidInput(format!(
                "Data too large: {data_len} bytes (max 31)"
            )));
        }

        // Calculate total bits: 8 bits for category+length + data bits + 16 bits for port
        let total_bits = 8 + (data.len() * 8) + 16;

        // Determine number of words needed
        // Each word encodes 12 bits
        // For IPv6: minimum 6 words (72 bits), can be 9 (108 bits) or 12 (144 bits)
        let words_needed = if total_bits <= 72 {
            6 // 6 words for simple patterns
        } else if total_bits <= 108 {
            9 // 9 words for medium complexity addresses
        } else {
            12 // 12 words for complex addresses
        };

        // Calculate padding needed (for potential future use)
        let bits_to_encode: usize = words_needed * 12;
        let _padding_bits = bits_to_encode.saturating_sub(total_bits);

        // For large data or when using 12 words, use byte array approach to avoid overflow
        if data.len() >= 14 || words_needed >= 12 {
            return self.encode_large_data_to_groups(data, port, words_needed, category);
        }

        // Build the number: category+length (8 bits) + data + port (16 bits)
        let mut n = 0u128;

        // Put category (3 bits) and data length (5 bits) in the lowest 8 bits
        let category_and_length = (category.to_bits() << 5) | (data_len & 0x1F);
        n |= category_and_length as u128;

        // Put compressed data in the next bits
        for (i, &byte) in data.iter().enumerate() {
            n |= (byte as u128) << (8 + (i * 8));
        }

        // Add port in the next 16 bits (after length + data)
        n |= (port as u128) << (8 + (data.len() * 8));

        // Extract words using modulo (similar to IPv4 approach)
        let mut word_indices = Vec::with_capacity(words_needed);
        let mut remaining = n;

        for _ in 0..words_needed {
            let index = (remaining % 4096) as u16;
            word_indices.push(index);
            remaining /= 4096;
        }

        // Always create groups of 4 words, padding as needed
        for chunk in word_indices.chunks(4) {
            let words: Result<Vec<String>> = chunk
                .iter()
                .map(|&idx| {
                    DICTIONARY
                        .get_word(idx)
                        .ok_or(FourWordError::InvalidWordIndex(idx))
                        .map(|s| s.to_string())
                })
                .collect();

            let words = words?;
            groups.push(FourWordGroup::new(
                words.first().cloned().unwrap_or_default(),
                words.get(1).cloned().unwrap_or_default(),
                words.get(2).cloned().unwrap_or_default(),
                words.get(3).cloned().unwrap_or_default(),
            ));
        }

        Ok(groups)
    }

    /// Encodes large data (>14 bytes) using byte array approach to avoid overflow
    fn encode_large_data_to_groups(
        &self,
        data: &[u8],
        port: u16,
        words_needed: usize,
        category: Ipv6Category,
    ) -> Result<Vec<FourWordGroup>> {
        // Create a byte array with category+length prefix, data, and port
        let mut all_bytes = Vec::new();
        let data_len = data.len() as u8;
        if data_len > 31 {
            return Err(FourWordError::InvalidInput(format!(
                "Data too large: {data_len} bytes (max 31)"
            )));
        }

        // Pack category (3 bits) and length (5 bits) into first byte
        let category_and_length = (category.to_bits() << 5) | (data_len & 0x1F);
        all_bytes.push(category_and_length); // Category+length prefix
        all_bytes.extend_from_slice(data); // Data
        all_bytes.extend_from_slice(&port.to_le_bytes()); // Port in little-endian

        // Pad to required number of bytes
        let bytes_needed = (words_needed * 12).div_ceil(8); // Round up to bytes
        while all_bytes.len() < bytes_needed {
            all_bytes.push(0);
        }

        // Extract word indices directly from byte array
        let mut word_indices = Vec::with_capacity(words_needed);

        // Extract 12-bit chunks as word indices
        let mut bit_offset = 0;
        for _ in 0..words_needed {
            let mut word_index = 0u16;

            // Extract 12 bits starting at bit_offset
            for bit in 0..12 {
                let byte_idx = (bit_offset + bit) / 8;
                let bit_idx = (bit_offset + bit) % 8;

                if byte_idx < all_bytes.len() {
                    let bit_val = (all_bytes[byte_idx] >> bit_idx) & 1;
                    word_index |= (bit_val as u16) << bit;
                }
            }

            // Ensure word index is within dictionary range
            word_index %= 4096;
            word_indices.push(word_index);
            bit_offset += 12;
        }

        // Create groups of 4 words
        let mut groups = Vec::new();
        for chunk in word_indices.chunks(4) {
            let words: Result<Vec<String>> = chunk
                .iter()
                .map(|&idx| {
                    DICTIONARY
                        .get_word(idx)
                        .ok_or(FourWordError::InvalidWordIndex(idx))
                        .map(|s| s.to_string())
                })
                .collect();

            let words = words?;
            groups.push(FourWordGroup::new(
                words.first().cloned().unwrap_or_default(),
                words.get(1).cloned().unwrap_or_default(),
                words.get(2).cloned().unwrap_or_default(),
                words.get(3).cloned().unwrap_or_default(),
            ));
        }

        Ok(groups)
    }

    /// Decodes groups of words back to bytes, port, and actual category
    fn decode_groups_to_bytes(
        &self,
        groups: &[FourWordGroup],
        _encoding_category: Ipv6Category,
    ) -> Result<(Vec<u8>, u16, Ipv6Category)> {
        // Flatten all words from groups
        let mut all_words = Vec::new();
        for group in groups {
            all_words.extend_from_slice(group.words());
        }

        // Filter out empty words and special markers (from potential padding)
        let all_words: Vec<&String> = all_words
            .iter()
            .filter(|w| !w.is_empty() && !w.starts_with("__MARKER_"))
            .collect();

        // For large encodings (12 words), use byte array approach
        if all_words.len() >= 12 {
            return self.decode_large_data_from_groups(&all_words);
        }

        // Reconstruct the number using iterative multiplication to avoid overflow
        let mut n = 0u128;
        let mut base = 1u128;

        for word in all_words.iter() {
            let index = DICTIONARY
                .get_index(word)
                .ok_or_else(|| FourWordError::InvalidWord(word.to_string()))?;

            // Check for potential overflow before multiplication
            if let Some(contribution) = base.checked_mul(index as u128) {
                n = n.checked_add(contribution).ok_or_else(|| {
                    FourWordError::InvalidInput("Numeric overflow in decoding".to_string())
                })?;

                // Update base for next iteration, but stop if it would overflow
                if base.checked_mul(4096).is_none() {
                    break;
                }
                base *= 4096;
            } else {
                return Err(FourWordError::InvalidInput(
                    "Numeric overflow in decoding".to_string(),
                ));
            }
        }

        // Extract category (3 bits) and data length (5 bits) from the lowest 8 bits
        let category_and_length = (n & 0xFF) as u8;
        let data_len = (category_and_length & 0x1F) as usize; // Lower 5 bits
        let decoded_category_bits = (category_and_length >> 5) & 0x07; // Upper 3 bits

        // Extract compressed data from the next data_len bytes
        let mut compressed_bytes = Vec::new();
        for i in 0..data_len {
            let byte = ((n >> (8 + (i * 8))) & 0xFF) as u8;
            compressed_bytes.push(byte);
        }

        // Extract port from the next 16 bits
        let port = ((n >> (8 + (data_len * 8))) & 0xFFFF) as u16;

        // Decode the actual category from the bits
        let actual_category = Ipv6Category::from_bits(decoded_category_bits)?;

        // Special handling for GlobalUnicast with provider patterns
        // If the decoded category is GlobalUnicast and we have 13 bytes,
        // the first byte is a pattern ID, not part of the category/length encoding
        if actual_category == Ipv6Category::GlobalUnicast && compressed_bytes.len() == 13 {
            // This is a provider pattern encoding - prepend the category bits
            // to match what the decompressor expects
            Ok((compressed_bytes, port, actual_category))
        } else {
            Ok((compressed_bytes, port, actual_category))
        }
    }

    /// Decodes large data (12 words) using byte array approach to avoid overflow
    fn decode_large_data_from_groups(
        &self,
        all_words: &[&String],
    ) -> Result<(Vec<u8>, u16, Ipv6Category)> {
        // Convert words back to indices
        let mut word_indices = Vec::new();
        for word in all_words {
            let index = DICTIONARY
                .get_index(word)
                .ok_or_else(|| FourWordError::InvalidWord(word.to_string()))?;
            word_indices.push(index);
        }

        // Convert word indices back to byte array
        let bytes_needed = (all_words.len() * 12).div_ceil(8); // Round up to bytes
        let mut all_bytes = vec![0u8; bytes_needed];

        // Pack 12-bit word indices into byte array
        let mut bit_offset = 0;
        for &word_index in word_indices.iter() {
            // Pack 12 bits starting at bit_offset
            for bit in 0..12 {
                let byte_idx = (bit_offset + bit) / 8;
                let bit_idx = (bit_offset + bit) % 8;

                if byte_idx < all_bytes.len() {
                    let bit_val = (word_index >> bit) & 1;
                    if bit_val == 1 {
                        all_bytes[byte_idx] |= 1 << bit_idx;
                    }
                }
            }
            bit_offset += 12;
        }

        // Extract category (3 bits) and data length (5 bits) from first byte
        let category_and_length = all_bytes[0];
        let data_len = (category_and_length & 0x1F) as usize; // Lower 5 bits
        let decoded_category_bits = (category_and_length >> 5) & 0x07; // Upper 3 bits

        // Extract compressed data
        let compressed_bytes = all_bytes[1..1 + data_len].to_vec();

        // Extract port from little-endian bytes
        let port_start = 1 + data_len;
        let port = if port_start + 2 <= all_bytes.len() {
            u16::from_le_bytes([all_bytes[port_start], all_bytes[port_start + 1]])
        } else {
            // If we can't read a full 2-byte port, use the special marker for "no port specified"
            65535
        };

        // Decode the actual category from the bits
        let actual_category = Ipv6Category::from_bits(decoded_category_bits)?;

        Ok((compressed_bytes, port, actual_category))
    }
}

impl Default for FourWordIpv6Encoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn test_encode_decode_ipv6() {
        let encoder = FourWordIpv6Encoder::new();

        let test_cases = vec![
            "[::1]:443",
            "[::]:80",
            "[fe80::1]:22",
            "[2001:db8::1]:8080",
            "[2001:db8:85a3::8a2e:370:7334]:443",
            "[2001:4860:4860::8888]:53", // Google DNS
        ];

        for addr_str in test_cases {
            let addr: SocketAddr = addr_str.parse().unwrap();
            if let SocketAddr::V6(v6) = addr {
                println!("\nTesting: {addr_str}");
                let encoded = encoder.encode(&v6).unwrap();
                println!("Encoded: {} ({} words)", encoded, encoded.word_count());
                println!("Category: {:?}", encoded.category());
                let decoded = encoder.decode(&encoded).unwrap();
                println!("Decoded: {decoded}");

                assert_eq!(v6.ip(), decoded.ip(), "IP mismatch for {addr_str}");
                assert_eq!(v6.port(), decoded.port(), "Port mismatch for {addr_str}");
            }
        }
    }

    #[test]
    fn test_word_group_formatting() {
        let group = FourWordGroup::new(
            "hello".to_string(),
            "world".to_string(),
            "test".to_string(),
            "data".to_string(),
        );

        assert_eq!(group.to_string(), "hello world test data");
    }

    #[test]
    fn test_ipv6_encoding_formatting() {
        let groups = vec![
            FourWordGroup::new(
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
                "four".to_string(),
            ),
            FourWordGroup::new(
                "five".to_string(),
                "six".to_string(),
                "seven".to_string(),
                "eight".to_string(),
            ),
        ];

        let encoding = Ipv6FourWordGroupEncoding::new(groups, Ipv6Category::Loopback);

        assert_eq!(
            encoding.to_string(),
            "one two three four five six seven eight"
        );
        assert_eq!(
            encoding.to_dashed_string(),
            "one-two-three-four-five-six-seven-eight"
        );
        assert_eq!(encoding.word_count(), 8);
    }
}
