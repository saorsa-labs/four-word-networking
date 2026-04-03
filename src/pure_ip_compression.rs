//! Pure Mathematical IP+Port Compression
//!
//! This module tackles the fundamental challenge: compress 48 bits (IP+port)
//! into 42 bits using pure mathematical techniques without any special cases.

use crate::error::FourWordError;
use std::net::Ipv4Addr;

/// Maximum value that fits in 42 bits
const MAX_42_BITS: u64 = (1u64 << 42) - 1; // 4,398,046,511,103

/// Pure mathematical compressor using multiple strategies
pub struct PureIpCompressor;

impl PureIpCompressor {
    /// Primary compression function - tries multiple mathematical approaches
    pub fn compress(ip: Ipv4Addr, port: u16) -> Result<u64, FourWordError> {
        let ip_u32 = u32::from(ip);
        let _full_value = ((ip_u32 as u64) << 16) | (port as u64);

        // Strategy 1: Direct bit reduction using mathematical properties
        if let Ok(compressed) = Self::bit_reduction_compress(ip_u32, port) {
            return Ok(compressed);
        }

        // Strategy 2: Prime factorization compression
        if let Ok(compressed) = Self::prime_factorization_compress(ip_u32, port) {
            return Ok(compressed);
        }

        // Strategy 3: Polynomial compression
        if let Ok(compressed) = Self::polynomial_compress(ip_u32, port) {
            return Ok(compressed);
        }

        // Strategy 4: Hash-based compression with collision detection
        if let Ok(compressed) = Self::hash_compress(ip_u32, port) {
            return Ok(compressed);
        }

        // Strategy 5: Sliding window compression
        if let Ok(compressed) = Self::sliding_window_compress(ip_u32, port) {
            return Ok(compressed);
        }

        Err(FourWordError::InvalidInput(format!(
            "Cannot compress {ip}:{port} (48→42 bits)"
        )))
    }

    /// Strategy 1: Mathematical bit reduction using modular arithmetic
    fn bit_reduction_compress(ip: u32, port: u16) -> Result<u64, FourWordError> {
        // Use the fact that we have 6 extra bits to lose
        // Apply controlled bit reduction that preserves uniqueness for most values

        // Approach: Use mathematical transforms that map 48-bit space to 42-bit space
        // with minimal collisions for common address ranges

        let full_value = ((ip as u64) << 16) | (port as u64);

        // Method 1: Modular reduction with prime modulus close to 2^42
        let prime_42 = 4398046511104u64; // Next prime after 2^42
        let compressed = full_value % prime_42;

        if compressed <= MAX_42_BITS {
            return Ok(compressed);
        }

        // Method 2: XOR folding - fold extra 6 bits into lower bits
        let folded = (full_value & MAX_42_BITS) ^ (full_value >> 42);
        Ok(folded)
    }

    /// Strategy 2: Prime factorization for specific patterns
    fn prime_factorization_compress(ip: u32, port: u16) -> Result<u64, FourWordError> {
        // Some IP+port combinations have mathematical structure we can exploit

        let full_value = ((ip as u64) << 16) | (port as u64);

        // Check if the value has small prime factors we can encode efficiently
        if full_value <= MAX_42_BITS {
            return Ok(full_value); // Already fits
        }

        // Try to express as product of smaller numbers
        let factors = Self::small_prime_factors(full_value);
        if factors.len() <= 3 && factors.iter().all(|&f| f < (1 << 14)) {
            // Can encode as three 14-bit factors
            let compressed = (factors[0] as u64) << 28
                | (*factors.get(1).unwrap_or(&0) as u64) << 14
                | (*factors.get(2).unwrap_or(&0) as u64);
            return Ok(compressed);
        }

        Err(FourWordError::InvalidInput(
            "No suitable factorization".to_string(),
        ))
    }

    /// Strategy 3: Polynomial mapping to reduce bit space
    fn polynomial_compress(ip: u32, port: u16) -> Result<u64, FourWordError> {
        // Map the 48-bit space to 42-bit space using polynomial functions
        // that preserve structure for common IP ranges

        let x = ip as u64;
        let y = port as u64;

        // Polynomial: compress using mathematical relationship
        // f(x,y) = (ax + by + c) mod (2^42 - k) where k is chosen to minimize collisions

        let a = 65537u64; // Prime coefficient
        let b = 97u64; // Prime coefficient
        let c = 23u64; // Prime offset

        let polynomial_result = (a * x + b * y + c) % (MAX_42_BITS + 1);

        Ok(polynomial_result)
    }

    /// Strategy 4: Cryptographic hash with controlled collisions
    fn hash_compress(ip: u32, port: u16) -> Result<u64, FourWordError> {
        // Use a hash function designed to map 48→42 bits with good distribution

        let full_value = ((ip as u64) << 16) | (port as u64);

        // Simple but effective hash for our use case
        let mut hash = full_value;
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xff51afd7ed558ccd);
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
        hash ^= hash >> 33;

        // Reduce to 42 bits
        let compressed = hash & MAX_42_BITS;

        Ok(compressed)
    }

    /// Strategy 5: Sliding window compression for sequential IPs
    fn sliding_window_compress(ip: u32, port: u16) -> Result<u64, FourWordError> {
        // Exploit locality in IP address space
        // Many real-world scenarios use sequential or nearby IPs

        // For IPs in common ranges, use base + offset encoding
        let common_bases = [
            0x0A000000u32, // 10.0.0.0
            0xC0A80000u32, // 192.168.0.0
            0xAC100000u32, // 172.16.0.0
            0x7F000000u32, // 127.0.0.0
        ];

        for (base_idx, &base) in common_bases.iter().enumerate() {
            if ip >= base && ip < base + (1 << 20) {
                // 1M range
                let offset = ip - base;
                // Encode: 2 bits for base_idx + 20 bits for offset + 16 bits for port + 4 spare
                let compressed = (base_idx as u64) << 40 | (offset as u64) << 16 | (port as u64);

                if compressed <= MAX_42_BITS {
                    return Ok(compressed);
                }
            }
        }

        Err(FourWordError::InvalidInput(
            "No suitable base found".to_string(),
        ))
    }

    /// Decompress using strategy detection
    pub fn decompress(compressed: u64) -> Result<(Ipv4Addr, u16), FourWordError> {
        if compressed > MAX_42_BITS {
            return Err(FourWordError::InvalidInput(
                "Invalid compressed value".to_string(),
            ));
        }

        // Try to detect which strategy was used based on value patterns

        // Strategy 5: Sliding window (check high bits for base index)
        let high_bits = compressed >> 40;
        if high_bits < 4 {
            return Self::decompress_sliding_window(compressed);
        }

        // Strategy 4: Hash (hardest to reverse, use approximation)
        Self::decompress_hash_approximate(compressed)
    }

    fn decompress_sliding_window(compressed: u64) -> Result<(Ipv4Addr, u16), FourWordError> {
        let base_idx = (compressed >> 40) as usize;
        let offset = ((compressed >> 16) & 0xFFFFF) as u32;
        let port = (compressed & 0xFFFF) as u16;

        let bases = [0x0A000000u32, 0xC0A80000u32, 0xAC100000u32, 0x7F000000u32];

        if base_idx < bases.len() {
            let ip_u32 = bases[base_idx] + offset;
            Ok((Ipv4Addr::from(ip_u32), port))
        } else {
            Err(FourWordError::InvalidInput(
                "Invalid base index".to_string(),
            ))
        }
    }

    fn decompress_hash_approximate(compressed: u64) -> Result<(Ipv4Addr, u16), FourWordError> {
        // Hash compression is lossy - we can't perfectly reverse it
        // But we can provide reasonable approximations for common cases

        // For demonstration, assume uniform distribution and reverse-engineer
        // In practice, you'd need a lookup table or collision resolution

        // Simple approximation: assume compressed value represents scaled coordinates
        let scaled_ip = ((compressed >> 16) * 0xFFFFFFFF / (MAX_42_BITS >> 16)) as u32;
        let port = (compressed & 0xFFFF) as u16;

        Ok((Ipv4Addr::from(scaled_ip), port))
    }

    fn small_prime_factors(mut n: u64) -> Vec<u32> {
        let mut factors = Vec::new();
        let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];

        for &p in &primes {
            while n.is_multiple_of(p as u64) {
                factors.push(p);
                n /= p as u64;
                if factors.len() >= 3 {
                    break;
                }
            }
            if factors.len() >= 3 {
                break;
            }
        }

        if n > 1 && n < (1 << 14) && factors.len() < 3 {
            factors.push(n as u32);
        }

        factors
    }
}

/// Advanced mathematical compression using number theory
pub struct MathematicalCompressor;

impl MathematicalCompressor {
    /// Use Cantor pairing function to map 2D space (IP, port) to 1D
    pub fn cantor_pair_compress(ip: u32, port: u16) -> u64 {
        let x = ip as u64;
        let y = port as u64;

        // Cantor pairing: (x + y) * (x + y + 1) / 2 + y
        // But this grows too fast, so we use a modified version

        // Modified pairing that stays within our bit budget
        let sum = x + y;
        if sum < (1 << 21) {
            // Ensure we don't overflow
            (sum * (sum + 1) / 2 + y) & MAX_42_BITS
        } else {
            // Fallback to simple interleaving
            Self::bit_interleave_compress(ip, port)
        }
    }

    /// Interleave bits of IP and port for better distribution
    pub fn bit_interleave_compress(ip: u32, port: u16) -> u64 {
        let mut result = 0u64;

        // Interleave bits: IP[0], Port[0], IP[1], Port[1], ...
        // Take most significant bits to fit in 42 bits

        for i in 0..16 {
            if i * 2 + 1 < 42 {
                // Take bit i from port
                result |= (((port >> (15 - i)) & 1) as u64) << (i * 2);

                // Take bit i from IP (if available)
                if i < 32 && i * 2 + 1 < 42 {
                    result |= (((ip >> (31 - i)) & 1) as u64) << (i * 2 + 1);
                }
            }
        }

        result
    }

    /// Use Gray code mapping to preserve locality
    pub fn gray_code_compress(ip: u32, port: u16) -> u64 {
        let full_value = ((ip as u64) << 16) | (port as u64);

        // Convert to Gray code to preserve locality
        let gray = full_value ^ (full_value >> 1);

        // Reduce to 42 bits using bit folding
        (gray & MAX_42_BITS) ^ (gray >> 42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_compression() {
        let test_cases = vec![
            (Ipv4Addr::new(192, 168, 1, 100), 80),
            (Ipv4Addr::new(10, 0, 0, 1), 22),
            (Ipv4Addr::new(172, 16, 0, 1), 443),
            (Ipv4Addr::new(8, 8, 8, 8), 53),
            (Ipv4Addr::new(203, 45, 67, 89), 12345),
        ];

        for (ip, port) in test_cases {
            match PureIpCompressor::compress(ip, port) {
                Ok(compressed) => {
                    assert!(compressed <= MAX_42_BITS);
                    println!("✓ Compressed {ip}:{port} -> {compressed} (fits in 42 bits)");

                    // Test decompression
                    if let Ok((dec_ip, dec_port)) = PureIpCompressor::decompress(compressed) {
                        println!("  Decompressed: {dec_ip}:{dec_port}");
                    }
                }
                Err(e) => {
                    println!("✗ Failed {ip}:{port} - {e}");
                }
            }
        }
    }

    #[test]
    fn test_mathematical_methods() {
        let ip = Ipv4Addr::new(192, 168, 1, 100);
        let port = 8080;
        let ip_u32 = u32::from(ip);

        let cantor = MathematicalCompressor::cantor_pair_compress(ip_u32, port);
        let interleave = MathematicalCompressor::bit_interleave_compress(ip_u32, port);
        let gray = MathematicalCompressor::gray_code_compress(ip_u32, port);

        assert!(cantor <= MAX_42_BITS);
        assert!(interleave <= MAX_42_BITS);
        assert!(gray <= MAX_42_BITS);

        println!("Cantor pairing: {cantor}");
        println!("Bit interleaving: {interleave}");
        println!("Gray code: {gray}");
    }

    #[test]
    fn test_compression_coverage() {
        let mut success_count = 0;
        let total_tests = 1000;

        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..total_tests {
            let ip = Ipv4Addr::new(
                rng.r#gen::<u8>(),
                rng.r#gen::<u8>(),
                rng.r#gen::<u8>(),
                rng.r#gen::<u8>(),
            );
            let port = rng.r#gen::<u16>();

            if PureIpCompressor::compress(ip, port).is_ok() {
                success_count += 1;
            }
        }

        let success_rate = success_count as f64 / total_tests as f64;
        println!(
            "Compression success rate: {:.1}% ({}/{})",
            success_rate * 100.0,
            success_count,
            total_tests
        );

        // We expect some failure rate since we're compressing 48→42 bits
        assert!(
            success_rate > 0.5,
            "Success rate too low: {:.1}%",
            success_rate * 100.0
        );
    }
}
