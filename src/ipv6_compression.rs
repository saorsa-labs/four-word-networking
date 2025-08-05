//! IPv6 Hierarchical Compression Engine
//!
//! This module implements advanced compression techniques specifically designed
//! for IPv6 addresses, taking advantage of their hierarchical structure and
//! common patterns to achieve optimal compression ratios.

use crate::error::FourWordError;
use std::net::Ipv6Addr;

/// IPv6 address categories for compression optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ipv6Category {
    /// ::1 - IPv6 loopback (4 words)
    Loopback,
    /// fe80::/10 - Link-local addresses (3-4 words)
    LinkLocal,
    /// fc00::/7 - Unique local addresses (4-5 words)
    UniqueLocal,
    /// 2001:db8::/32 - Documentation addresses (4 words)
    Documentation,
    /// 2000::/3 - Global unicast (4-6 words)
    GlobalUnicast,
    /// ::/128 - Unspecified address (4 words)
    Unspecified,
    /// Multicast and other special addresses (5-6 words)
    Special,
}

impl Ipv6Category {
    /// Convert category to a 3-bit numeric value for encoding
    pub fn to_bits(&self) -> u8 {
        match self {
            Ipv6Category::Loopback => 0,
            Ipv6Category::LinkLocal => 1,
            Ipv6Category::UniqueLocal => 2,
            Ipv6Category::Documentation => 3,
            Ipv6Category::GlobalUnicast => 4,
            Ipv6Category::Unspecified => 5,
            Ipv6Category::Special => 6,
        }
    }

    /// Convert 3-bit numeric value back to category
    pub fn from_bits(bits: u8) -> Result<Self, FourWordError> {
        match bits {
            0 => Ok(Ipv6Category::Loopback),
            1 => Ok(Ipv6Category::LinkLocal),
            2 => Ok(Ipv6Category::UniqueLocal),
            3 => Ok(Ipv6Category::Documentation),
            4 => Ok(Ipv6Category::GlobalUnicast),
            5 => Ok(Ipv6Category::Unspecified),
            6 => Ok(Ipv6Category::Special),
            _ => Err(FourWordError::InvalidInput(format!(
                "Invalid category bits: {bits}"
            ))),
        }
    }
}

/// Compressed representation of an IPv6 address
#[derive(Debug, Clone)]
pub struct CompressedIpv6 {
    pub category: Ipv6Category,
    pub compressed_data: Vec<u8>,
    pub original_bits: usize,
    pub compressed_bits: usize,
    pub port: Option<u16>,
}

impl CompressedIpv6 {
    /// Creates compressed IPv6 from bytes and category
    pub fn from_bytes(data: &[u8], category: Ipv6Category) -> Result<Self, FourWordError> {
        if data.is_empty() {
            return Err(FourWordError::InvalidInput(
                "Empty compressed data".to_string(),
            ));
        }

        Ok(CompressedIpv6 {
            category,
            compressed_data: data.to_vec(),
            original_bits: 128,
            compressed_bits: data.len() * 8,
            port: None,
        })
    }

    /// Get bytes representation
    pub fn as_bytes(&self) -> Vec<u8> {
        self.compressed_data.clone()
    }

    /// Get the total compressed size including port
    pub fn total_bits(&self) -> usize {
        self.compressed_bits + self.port.map_or(0, |_| 16)
    }

    /// Get the recommended word count for this compression
    /// IPv6 always uses 4-6 words to distinguish from IPv4
    pub fn recommended_word_count(&self) -> usize {
        let total_bits = self.total_bits();
        // IPv6 always uses at least 4 words for clear differentiation from IPv4
        if total_bits <= 56 {
            4
        } else if total_bits <= 70 {
            5
        } else {
            6
        }
    }

    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        let original_total = self.original_bits + self.port.map_or(0, |_| 16);
        1.0 - (self.total_bits() as f64 / original_total as f64)
    }

    /// Get human-readable category description
    pub fn category_description(&self) -> &'static str {
        match self.category {
            Ipv6Category::Loopback => "IPv6 Loopback (::1)",
            Ipv6Category::LinkLocal => "Link-Local (fe80::)",
            Ipv6Category::UniqueLocal => "Unique Local (fc00::)",
            Ipv6Category::Documentation => "Documentation (2001:db8::)",
            Ipv6Category::GlobalUnicast => "Global Unicast",
            Ipv6Category::Unspecified => "Unspecified (::)",
            Ipv6Category::Special => "Special/Multicast",
        }
    }
}

/// Advanced IPv6 compression engine
pub struct Ipv6Compressor;

impl Default for Ipv6Compressor {
    fn default() -> Self {
        Ipv6Compressor
    }
}

impl Ipv6Compressor {
    /// Creates a new IPv6 compressor
    pub fn new() -> Self {
        Self
    }

    /// Compress an IPv6 address with optional port
    pub fn compress(
        &self,
        ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        let category = Self::categorize_address(&ip);

        match category {
            Ipv6Category::Loopback => Self::compress_loopback(ip, port),
            Ipv6Category::LinkLocal => Self::compress_link_local(ip, port),
            Ipv6Category::UniqueLocal => Self::compress_unique_local(ip, port),
            Ipv6Category::Documentation => Self::compress_documentation(ip, port),
            Ipv6Category::GlobalUnicast => Self::compress_global_unicast(ip, port),
            Ipv6Category::Unspecified => Self::compress_unspecified(ip, port),
            Ipv6Category::Special => Self::compress_special(ip, port),
        }
    }

    /// Decompress back to IPv6 address and port
    pub fn decompress(
        &self,
        compressed: &CompressedIpv6,
    ) -> Result<(Ipv6Addr, Option<u16>), FourWordError> {
        let ip = match compressed.category {
            Ipv6Category::Loopback => Self::decompress_loopback(&compressed.compressed_data)?,
            Ipv6Category::LinkLocal => Self::decompress_link_local(&compressed.compressed_data)?,
            Ipv6Category::UniqueLocal => {
                Self::decompress_unique_local(&compressed.compressed_data)?
            }
            Ipv6Category::Documentation => {
                Self::decompress_documentation(&compressed.compressed_data)?
            }
            Ipv6Category::GlobalUnicast => {
                Self::decompress_global_unicast(&compressed.compressed_data)?
            }
            Ipv6Category::Unspecified => Self::decompress_unspecified(&compressed.compressed_data)?,
            Ipv6Category::Special => Self::decompress_special(&compressed.compressed_data)?,
        };

        Ok((ip, compressed.port))
    }

    /// Categorize an IPv6 address for optimal compression
    fn categorize_address(ip: &Ipv6Addr) -> Ipv6Category {
        let segments = ip.segments();

        // Check for loopback ::1
        if ip.is_loopback() {
            return Ipv6Category::Loopback;
        }

        // Check for unspecified ::
        if ip.is_unspecified() {
            return Ipv6Category::Unspecified;
        }

        // Check for link-local fe80::/10
        if segments[0] & 0xFFC0 == 0xFE80 {
            return Ipv6Category::LinkLocal;
        }

        // Check for unique local fc00::/7
        if segments[0] & 0xFE00 == 0xFC00 {
            return Ipv6Category::UniqueLocal;
        }

        // Check for documentation 2001:db8::/32
        if segments[0] == 0x2001 && segments[1] == 0x0DB8 {
            return Ipv6Category::Documentation;
        }

        // Check for global unicast 2000::/3
        if segments[0] & 0xE000 == 0x2000 {
            return Ipv6Category::GlobalUnicast;
        }

        // Everything else (multicast, etc.)
        Ipv6Category::Special
    }

    /// Compress loopback address ::1
    fn compress_loopback(
        _ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        // Loopback is just ::1, but we ensure 4 words minimum for IPv6
        // Add padding bytes to ensure we reach 4 words (56 bits total)
        let padding = vec![0x00, 0x00, 0x01, 0x00, 0x00, 0x00]; // 48 bits of padding
        Ok(CompressedIpv6 {
            category: Ipv6Category::Loopback,
            compressed_data: padding,
            original_bits: 128,
            compressed_bits: 48, // Ensure 4 words minimum
            port,
        })
    }

    /// Compress link-local address fe80::/10
    fn compress_link_local(
        ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        let segments = ip.segments();

        // Link-local: fe80:0000:0000:0000:xxxx:xxxx:xxxx:xxxx
        // Optimize for common patterns

        // Check for simple patterns (fe80::1, fe80::2, etc.)
        let non_zero_segments: Vec<(usize, u16)> = segments[4..8]
            .iter()
            .enumerate()
            .filter(|&(_, &seg)| seg != 0)
            .map(|(i, &seg)| (i + 4, seg))
            .collect();

        let mut compressed = Vec::new();
        let compressed_bits;

        if non_zero_segments.is_empty() {
            // fe80:: - all zeros in interface ID
            // Use 6 bytes to match loopback and other simple patterns
            compressed = vec![0, 0, 0, 0, 0, 0]; // Marker + padding for 48 bits
            compressed_bits = 48; // 6 bytes
        } else if non_zero_segments.len() == 1 && non_zero_segments[0].1 <= 255 {
            // Single small value like fe80::1 - store position + value
            let (pos, val) = non_zero_segments[0];
            // Use 6 bytes to match loopback and other simple patterns
            compressed = vec![1, (pos - 4) as u8, val as u8, 0, 0, 0]; // Marker + data + padding
            compressed_bits = 48; // 6 bytes
        } else if segments[4] & 0x0200 == 0x0200 && segments[7] == 0 {
            // EUI-64 derived address - only use this pattern if segment[7] is 0
            // since the reconstruction doesn't preserve segment[7]
            compressed = vec![2]; // Marker for EUI-64
            let mac_derived = [
                (segments[4] ^ 0x0200) as u8, // Remove universal/local bit
                (segments[4] >> 8) as u8,
                (segments[5]) as u8,
                (segments[5] >> 8) as u8,
                (segments[6]) as u8,
            ];
            compressed.extend_from_slice(&mac_derived);
            compressed_bits = 48; // 6 bytes total
        } else {
            // Complex pattern - store efficiently with RLE
            compressed.push(3); // Marker for complex pattern
            for &(pos, val) in &non_zero_segments {
                compressed.push((pos - 4) as u8);
                compressed.extend_from_slice(&val.to_be_bytes());
            }
            compressed.push(255); // End marker
            compressed_bits = 3 + (compressed.len() * 8); // category + data
        }

        Ok(CompressedIpv6 {
            category: Ipv6Category::LinkLocal,
            compressed_data: compressed,
            original_bits: 128,
            compressed_bits,
            port,
        })
    }

    /// Compress unique local address fc00::/7
    fn compress_unique_local(
        ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        let segments = ip.segments();

        // Unique local: fcxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx
        // ULA compression is always lossy - only preserve the first 64 bits (4 segments)
        // Interface ID (segments 4-7) is always dropped as per design
        let mut compressed = vec![];

        // Store only segments[0-3] as 8 bytes (prefix + global ID + subnet)
        compressed.extend_from_slice(&segments[0].to_be_bytes()); // segments[0] (includes fc/fd prefix)
        compressed.extend_from_slice(&segments[1].to_be_bytes()); // segments[1]
        compressed.extend_from_slice(&segments[2].to_be_bytes()); // segments[2]
        compressed.extend_from_slice(&segments[3].to_be_bytes()); // segments[3] (subnet)

        // ULA compression always uses only 64 bits (4 segments) + category
        let compressed_bits = 3 + 64; // category + 4 segments (8 bytes)

        Ok(CompressedIpv6 {
            category: Ipv6Category::UniqueLocal,
            compressed_data: compressed,
            original_bits: 128,
            compressed_bits,
            port,
        })
    }

    /// Compress documentation address 2001:db8::/32
    fn compress_documentation(
        ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        let segments = ip.segments();

        // Documentation: 2001:0db8:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx
        // For documentation addresses, we need to preserve interface ID segments
        // to avoid losing data like in 2001:db8:85a3::8a2e:370:7334

        let mut compressed = Vec::new();

        // Store segments 2-3 (routing prefix after 2001:db8)
        compressed.extend_from_slice(&segments[2].to_be_bytes());
        compressed.extend_from_slice(&segments[3].to_be_bytes());

        // Check for non-zero segments in the interface ID (segments 4-7)
        let non_zero_interface: Vec<(usize, u16)> = segments[4..8]
            .iter()
            .enumerate()
            .filter(|&(_, &seg)| seg != 0)
            .map(|(i, &seg)| (i + 4, seg))
            .collect();

        if non_zero_interface.is_empty() {
            // No interface ID - use marker for empty interface
            compressed.push(0); // Marker
        } else if non_zero_interface.len() == 1 && non_zero_interface[0].1 <= 255 {
            // Single small value in interface ID - compact encoding
            let (pos, val) = non_zero_interface[0];
            compressed.push(1); // Marker
            compressed.push((pos - 4) as u8); // Position in interface ID
            compressed.push(val as u8); // Value (single byte)
        } else {
            // Complex interface ID - store all non-zero segments with position
            compressed.push(2); // Marker for complex pattern
            for &(pos, val) in &non_zero_interface {
                compressed.push((pos - 4) as u8); // Position relative to interface ID start
                compressed.extend_from_slice(&val.to_be_bytes()); // Full 16-bit value
            }
            compressed.push(255); // End marker
        }

        // Variable length depending on complexity
        let compressed_bits = compressed.len() * 8;

        Ok(CompressedIpv6 {
            category: Ipv6Category::Documentation,
            compressed_data: compressed,
            original_bits: 128,
            compressed_bits,
            port,
        })
    }

    /// Compress global unicast address 2000::/3
    fn compress_global_unicast(
        ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        let segments = ip.segments();

        // Global unicast is the most challenging to compress
        // We'll use statistical compression based on common patterns

        // Check for common provider patterns
        if let Some(compressed) = Self::try_provider_patterns(&segments) {
            return Ok(CompressedIpv6 {
                category: Ipv6Category::GlobalUnicast,
                compressed_data: compressed,
                original_bits: 128,
                compressed_bits: 3 + 48, // category + pattern data
                port,
            });
        }

        // Fallback: store all segments (full 128 bits)
        let mut compressed = Vec::new();
        for segment in segments {
            compressed.extend_from_slice(&segment.to_be_bytes());
        }

        Ok(CompressedIpv6 {
            category: Ipv6Category::GlobalUnicast,
            compressed_data: compressed,
            original_bits: 128,
            compressed_bits: 3 + 128, // category + full address
            port,
        })
    }

    /// Compress unspecified address ::
    fn compress_unspecified(
        _ip: Ipv6Addr,
        port: Option<u16>,
    ) -> Result<CompressedIpv6, FourWordError> {
        // Unspecified is all zeros, but we ensure 4 words minimum for IPv6
        // Add padding bytes to ensure we reach 4 words (56 bits total)
        let padding = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 48 bits of padding
        Ok(CompressedIpv6 {
            category: Ipv6Category::Unspecified,
            compressed_data: padding,
            original_bits: 128,
            compressed_bits: 48, // Ensure 4 words minimum
            port,
        })
    }

    /// Compress special addresses (multicast, etc.)
    fn compress_special(ip: Ipv6Addr, port: Option<u16>) -> Result<CompressedIpv6, FourWordError> {
        let segments = ip.segments();

        // For special addresses, store all segments but mark as special
        let mut compressed = Vec::new();
        for segment in segments {
            compressed.extend_from_slice(&segment.to_be_bytes());
        }

        Ok(CompressedIpv6 {
            category: Ipv6Category::Special,
            compressed_data: compressed,
            original_bits: 128,
            compressed_bits: 3 + 128, // category + full address
            port,
        })
    }

    /// Try to compress using common provider patterns
    fn try_provider_patterns(segments: &[u16; 8]) -> Option<Vec<u8>> {
        // Common patterns from major IPv6 providers
        let patterns = [
            // Google: 2001:4860::/32
            ([0x2001, 0x4860], 32),
            // Hurricane Electric: 2001:470::/32
            ([0x2001, 0x0470], 32),
            // Comcast: 2001:558::/32
            ([0x2001, 0x0558], 32),
        ];

        for (pattern, prefix_bits) in patterns {
            if segments[0] == pattern[0] && segments[1] == pattern[1] {
                // Store pattern ID + remaining bits
                let pattern_id = match pattern {
                    [0x2001, 0x4860] => 0u8,
                    [0x2001, 0x0470] => 1u8,
                    [0x2001, 0x0558] => 2u8,
                    _ => continue,
                };

                let mut compressed = vec![pattern_id];

                // Store the remaining segments after the pattern
                let remaining_segments = 8 - (prefix_bits / 16);
                for segment in segments.iter().skip(8 - remaining_segments) {
                    compressed.extend_from_slice(&segment.to_be_bytes());
                }

                return Some(compressed);
            }
        }

        None
    }

    // Decompression methods (implementations would mirror compression logic)
    fn decompress_loopback(_data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        Ok(Ipv6Addr::LOCALHOST)
    }

    fn decompress_link_local(data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        if data.is_empty() {
            return Err(FourWordError::InvalidInput(
                "Empty link-local data".to_string(),
            ));
        }

        let mut segments = [0u16; 8];
        segments[0] = 0xfe80;
        segments[1] = 0x0000;
        segments[2] = 0x0000;
        segments[3] = 0x0000;

        match data[0] {
            0 => {
                // All zeros pattern: fe80::
                // segments already initialized correctly
            }
            1 => {
                // Single value pattern
                if data.len() >= 3 {
                    let pos = data[1] as usize + 4; // Convert back to absolute position
                    let val = data[2] as u16;
                    if (4..8).contains(&pos) {
                        segments[pos] = val;
                    }
                }
            }
            2 => {
                // EUI-64 derived address
                if data.len() >= 6 {
                    segments[4] = ((data[2] as u16) << 8) | (data[1] as u16) | 0x0200;
                    segments[5] = ((data[4] as u16) << 8) | (data[3] as u16);
                    segments[6] = data[5] as u16;
                    // segments[7] remains 0 - simplified reconstruction
                }
            }
            3 => {
                // Complex pattern with RLE
                let mut i = 1;
                while i < data.len() && data[i] != 255 {
                    if i + 2 < data.len() {
                        let pos = data[i] as usize + 4; // Convert back to absolute position
                        let val = ((data[i + 1] as u16) << 8) | (data[i + 2] as u16);
                        if (4..8).contains(&pos) {
                            segments[pos] = val;
                        }
                        i += 3;
                    } else {
                        break;
                    }
                }
            }
            _ => {
                return Err(FourWordError::InvalidInput(
                    "Invalid link-local pattern".to_string(),
                ));
            }
        }

        Ok(Ipv6Addr::from(segments))
    }

    fn decompress_unique_local(data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        if data.len() == 8 {
            // Interface ID is zero, only prefix + global ID + subnet are stored
            let segments = [
                ((data[0] as u16) << 8) | (data[1] as u16), // segments[0] (fc/fd prefix)
                ((data[2] as u16) << 8) | (data[3] as u16), // segments[1]
                ((data[4] as u16) << 8) | (data[5] as u16), // segments[2]
                ((data[6] as u16) << 8) | (data[7] as u16), // segments[3] (subnet)
                0x0000,                                     // segments[4] - interface ID is zero
                0x0000,                                     // segments[5] - interface ID is zero
                0x0000,                                     // segments[6] - interface ID is zero
                0x0000,                                     // segments[7] - interface ID is zero
            ];
            Ok(Ipv6Addr::from(segments))
        } else if data.len() == 16 {
            // Interface ID is non-zero, all 8 segments are stored
            let segments = [
                ((data[0] as u16) << 8) | (data[1] as u16), // segments[0] (fc/fd prefix)
                ((data[2] as u16) << 8) | (data[3] as u16), // segments[1]
                ((data[4] as u16) << 8) | (data[5] as u16), // segments[2]
                ((data[6] as u16) << 8) | (data[7] as u16), // segments[3] (subnet)
                ((data[8] as u16) << 8) | (data[9] as u16), // segments[4] - interface ID
                ((data[10] as u16) << 8) | (data[11] as u16), // segments[5] - interface ID
                ((data[12] as u16) << 8) | (data[13] as u16), // segments[6] - interface ID
                ((data[14] as u16) << 8) | (data[15] as u16), // segments[7] - interface ID
            ];
            Ok(Ipv6Addr::from(segments))
        } else {
            Err(FourWordError::InvalidInput(format!(
                "Invalid unique local data length: {} (expected 8 or 16 bytes)",
                data.len()
            )))
        }
    }

    fn decompress_documentation(data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        if data.len() < 5 {
            return Err(FourWordError::InvalidInput(
                "Documentation data too short - expected at least 5 bytes".to_string(),
            ));
        }

        let mut segments = [0u16; 8];
        segments[0] = 0x2001;
        segments[1] = 0x0db8;

        // Read segments 2-3 (routing prefix) from bytes 0-3
        segments[2] = ((data[0] as u16) << 8) | (data[1] as u16);
        segments[3] = ((data[2] as u16) << 8) | (data[3] as u16);

        // Read interface ID info starting from byte 4
        if data.len() <= 4 {
            return Ok(Ipv6Addr::from(segments)); // No interface ID data
        }

        let marker = data[4];
        let mut offset = 5;

        match marker {
            0 => {
                // No interface ID - segments 4-7 remain zero
            }
            1 => {
                // Single small value in interface ID
                if data.len() >= 7 {
                    let pos = data[5] as usize + 4; // Position in absolute terms
                    let val = data[6] as u16;
                    if (4..8).contains(&pos) {
                        segments[pos] = val;
                    }
                }
            }
            2 => {
                // Complex interface ID - read position/value pairs until end marker
                while offset < data.len() && data[offset] != 255 {
                    if offset + 2 < data.len() {
                        let pos = data[offset] as usize + 4; // Position in absolute terms
                        let val = ((data[offset + 1] as u16) << 8) | (data[offset + 2] as u16);
                        if (4..8).contains(&pos) {
                            segments[pos] = val;
                        }
                        offset += 3; // Move to next position/value pair
                    } else {
                        break;
                    }
                }
            }
            _ => {
                return Err(FourWordError::InvalidInput(format!(
                    "Invalid documentation marker: {marker}"
                )));
            }
        }

        Ok(Ipv6Addr::from(segments))
    }

    fn decompress_global_unicast(data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        if data.len() == 16 {
            // Fallback case: full 16 bytes (8 segments)
            let mut segments = [0u16; 8];
            for i in 0..8 {
                segments[i] = ((data[i * 2] as u16) << 8) | (data[i * 2 + 1] as u16);
            }
            Ok(Ipv6Addr::from(segments))
        } else if data.len() == 13 {
            // Provider pattern case: 1 byte pattern ID + 12 bytes (6 segments)
            let pattern_id = data[0];
            let mut segments = [0u16; 8];

            // Set the prefix based on pattern ID
            match pattern_id {
                0 => {
                    // Google: 2001:4860::/32
                    segments[0] = 0x2001;
                    segments[1] = 0x4860;
                }
                1 => {
                    // Hurricane Electric: 2001:470::/32
                    segments[0] = 0x2001;
                    segments[1] = 0x0470;
                }
                2 => {
                    // Comcast: 2001:558::/32
                    segments[0] = 0x2001;
                    segments[1] = 0x0558;
                }
                _ => {
                    return Err(FourWordError::InvalidInput(format!(
                        "Invalid provider pattern ID: {pattern_id}"
                    )))
                }
            }

            // Decode the remaining 6 segments from the 12 bytes
            for i in 0..6 {
                let byte_offset = 1 + (i * 2); // Skip pattern ID byte
                segments[i + 2] =
                    ((data[byte_offset] as u16) << 8) | (data[byte_offset + 1] as u16);
            }

            Ok(Ipv6Addr::from(segments))
        } else {
            Err(FourWordError::InvalidInput(format!(
                "Invalid global unicast data length: {} bytes",
                data.len()
            )))
        }
    }

    fn decompress_unspecified(_data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        Ok(Ipv6Addr::UNSPECIFIED)
    }

    fn decompress_special(data: &[u8]) -> Result<Ipv6Addr, FourWordError> {
        if data.len() >= 16 {
            let mut segments = [0u16; 8];
            for i in 0..8 {
                segments[i] = ((data[i * 2] as u16) << 8) | (data[i * 2 + 1] as u16);
            }
            Ok(Ipv6Addr::from(segments))
        } else {
            Err(FourWordError::InvalidInput(
                "Invalid special address data".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_loopback_compression() {
        let compressor = Ipv6Compressor::new();
        let ip = Ipv6Addr::LOCALHOST;
        let compressed = compressor.compress(ip, Some(443)).unwrap();

        assert_eq!(compressed.category, Ipv6Category::Loopback);
        assert_eq!(compressed.compressed_data.len(), 6); // Padded to 6 bytes
                                                         // With category byte + 6 bytes data = 56 bits total = 4 words
        assert!(compressed.recommended_word_count() >= 4); // IPv6 minimum 4 words

        let (decompressed_ip, port) = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed_ip, ip);
        assert_eq!(port, Some(443));
    }

    #[test]
    fn test_unspecified_compression() {
        let compressor = Ipv6Compressor::new();
        let ip = Ipv6Addr::UNSPECIFIED;
        let compressed = compressor.compress(ip, None).unwrap();

        assert_eq!(compressed.category, Ipv6Category::Unspecified);
        assert_eq!(compressed.compressed_data.len(), 6); // Padded to 6 bytes
        assert!(compressed.recommended_word_count() >= 4); // IPv6 minimum 4 words
    }

    #[test]
    fn test_link_local_compression() {
        let compressor = Ipv6Compressor::new();
        let ip = Ipv6Addr::from_str("fe80::1").unwrap();
        let compressed = compressor.compress(ip, Some(22)).unwrap();

        assert_eq!(compressed.category, Ipv6Category::LinkLocal);
        assert!(compressed.recommended_word_count() >= 4); // IPv6 minimum 4 words
        assert!(compressed.compression_ratio() > 0.3); // Adjusted for padding
    }

    #[test]
    fn test_documentation_compression() {
        let compressor = Ipv6Compressor::new();
        let ip = Ipv6Addr::from_str("2001:db8::1").unwrap();
        let compressed = compressor.compress(ip, Some(80)).unwrap();

        assert_eq!(compressed.category, Ipv6Category::Documentation);
        assert!(
            compressed.recommended_word_count() >= 4 && compressed.recommended_word_count() <= 6
        );
    }

    #[test]
    fn test_category_descriptions() {
        let compressor = Ipv6Compressor::new();
        let ip = Ipv6Addr::LOCALHOST;
        let compressed = compressor.compress(ip, None).unwrap();
        assert_eq!(compressed.category_description(), "IPv6 Loopback (::1)");
    }

    #[test]
    fn test_compression_ratios() {
        let test_cases = vec![
            (Ipv6Addr::LOCALHOST, "loopback"),
            (Ipv6Addr::UNSPECIFIED, "unspecified"),
            (Ipv6Addr::from_str("fe80::1").unwrap(), "link-local"),
        ];

        for (ip, name) in test_cases {
            let compressor = Ipv6Compressor::new();
            let compressed = compressor.compress(ip, Some(443)).unwrap();
            let ratio = compressed.compression_ratio();
            println!("{}: {:.1}% compression", name, ratio * 100.0);
            assert!(ratio > 0.0, "{name} should have some compression");
        }
    }
}
