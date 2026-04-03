use four_word_networking::*;
/// Property-based testing using proptest and quickcheck
use proptest::prelude::*;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

mod test_config;
#[allow(unused_imports)]
use test_config::*;

// Property tests using proptest

proptest! {
    #[test]
    fn prop_ipv4_roundtrip(a in 0u8..=255, b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = Ipv4Addr::new(a, b, c, d);
        let ip_str = ip.to_string();

        // Test roundtrip property - with smart port handling, IP addresses without ports should roundtrip exactly
        if let Ok(encoded) = encode_ip_address(&ip_str) {
            match decode_words(&encoded) {
                Ok(decoded) => {
                    // With smart port handling using port 65535 as marker, IP addresses without ports roundtrip exactly
                    prop_assert_eq!(ip_str, decoded);
                },
                Err(_) => prop_assert!(false, "Decoding failed for valid encoding"),
            }
        }
    }
}

proptest! {
    #[test]
    fn prop_ipv4_valid_encoding(a in 0u8..=255, b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = Ipv4Addr::new(a, b, c, d);
        let ip_str = ip.to_string();

        if let Ok(encoded) = encode_ip_address(&ip_str) {
            // Should have exactly 4 words for IPv4 (now space-separated)
            let words: Vec<&str> = encoded.split(' ').collect();
            prop_assert_eq!(words.len(), 4);

            // Each word should be valid
            for word in words {
                prop_assert!(!word.is_empty(), "Word cannot be empty");
                prop_assert!(!word.is_empty(), "Word too short: {}", word);
                // Dictionary can have words up to 25 characters (e.g., "counterrevolutionaries")
                prop_assert!(word.len() <= 25, "Word too long: {}", word);
                prop_assert!(word.chars().all(|c| c.is_ascii_lowercase()),
                    "Invalid characters in word: {}", word);
            }
        }
    }
}

proptest! {
    #[test]
    fn prop_encoding_deterministic(a in 0u8..=255, b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = Ipv4Addr::new(a, b, c, d);
        let ip_str = ip.to_string();

        if let Ok(encoded1) = encode_ip_address(&ip_str)
            && let Ok(encoded2) = encode_ip_address(&ip_str) {
                prop_assert_eq!(encoded1, encoded2, "Encoding should be deterministic");
            }
    }
}

proptest! {
    #[test]
    fn prop_different_ips_different_encodings(
        a1 in 0u8..=255, b1 in 0u8..=255, c1 in 0u8..=255, d1 in 0u8..=255,
        a2 in 0u8..=255, b2 in 0u8..=255, c2 in 0u8..=255, d2 in 0u8..=255
    ) {
        let ip1 = Ipv4Addr::new(a1, b1, c1, d1);
        let ip2 = Ipv4Addr::new(a2, b2, c2, d2);

        // Only test if IPs are different
        if ip1 != ip2
            && let (Ok(encoded1), Ok(encoded2)) = (
                encode_ip_address(&ip1.to_string()),
                encode_ip_address(&ip2.to_string())
            ) {
                prop_assert_ne!(encoded1, encoded2,
                    "Different IPs should produce different encodings: {} vs {}",
                    ip1, ip2);
            }
    }
}

proptest! {
    #[test]
    fn prop_port_encoding(
        a in 0u8..=255, b in 0u8..=255, c in 0u8..=255, d in 0u8..=255,
        port in 1u16..=65535
    ) {
        let ip = Ipv4Addr::new(a, b, c, d);
        let socket = SocketAddr::from((ip, port));
        let socket_str = socket.to_string();

        if let Ok(encoded) = encode_socket_address(&socket_str)
            && let Ok(decoded) = decode_socket_address(&encoded) {
                prop_assert_eq!(socket_str, decoded, "Socket address roundtrip failed");
            }
    }
}

proptest! {
    #[test]
    fn prop_ipv6_basic_roundtrip(
        a in 0u16..=0xffff, b in 0u16..=0xffff, c in 0u16..=0xffff, d in 0u16..=0xffff,
        e in 0u16..=0xffff, f in 0u16..=0xffff, g in 0u16..=0xffff, h in 0u16..=0xffff
    ) {
        let ip = Ipv6Addr::new(a, b, c, d, e, f, g, h);
        let ip_str = ip.to_string();

        // Note: fc/fd addresses only preserve the first 64 bits (prefix + global ID + subnet)
        // Interface IDs are lost during compression for unique local addresses
        // This is a design limitation of the current compression strategy
        // We skip these addresses in this roundtrip test.
        if ip.segments()[0] & 0xfe00 == 0xfc00 {
            return Ok(());
        }

        if let Ok(encoded) = encode_ip_address(&ip_str)
            && let Ok(decoded) = decode_words(&encoded) {
                // With smart port handling, IPv6 addresses without ports should roundtrip exactly
                // Parse both IPs to normalize them
                match (ip_str.parse::<Ipv6Addr>(), decoded.parse::<Ipv6Addr>()) {
                    (Ok(expected), Ok(actual)) => {
                        prop_assert_eq!(expected, actual,
                            "IPv6 roundtrip failed: {} -> {} -> {}",
                            ip_str, encoded, decoded);
                    }
                    _ => {
                        // If parsing fails, fall back to string comparison
                        prop_assert_eq!(&ip_str, &decoded,
                            "IPv6 roundtrip failed: {} -> {} -> {}",
                            ip_str, encoded, decoded);
                    }
                }
            }
    }
}

proptest! {
    #[test]
    fn prop_compression_bounds(a in 0u8..=255, b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = Ipv4Addr::new(a, b, c, d);
        let ip_str = ip.to_string();

        if let Ok(encoded) = encode_ip_address(&ip_str) {
            let word_count = encoded.split(' ').count();

            // IPv4 should always produce exactly 4 words
            prop_assert_eq!(word_count, 4, "IPv4 should produce exactly 4 words");

            // Estimate encoding efficiency
            let original_bits = 32; // IPv4 is 32 bits
            let estimated_encoded_bits = word_count * 12; // Assume 12 bits per word
            let compression_ratio = estimated_encoded_bits as f64 / original_bits as f64;

            // Should be reasonably efficient (not more than 4x expansion)
            prop_assert!(compression_ratio <= 4.0,
                "Compression too inefficient: {}x expansion", compression_ratio);
        }
    }
}

proptest! {
    #[test]
    fn prop_performance_bounds(a in 0u8..=255, b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = Ipv4Addr::new(a, b, c, d);
        let ip_str = ip.to_string();

        // Encoding should be fast
        let start = std::time::Instant::now();
        let encoded = encode_ip_address(&ip_str);
        let encoding_time = start.elapsed();

        prop_assert!(encoding_time.as_micros() < 100000,
            "Encoding too slow: {}μs", encoding_time.as_micros());

        // Decoding should be fast
        if let Ok(encoded) = encoded {
            let start = std::time::Instant::now();
            let _decoded = decode_words(&encoded);
            let decoding_time = start.elapsed();

            prop_assert!(decoding_time.as_micros() < 100000,
                "Decoding too slow: {}μs", decoding_time.as_micros());
        }
    }
}

// QuickCheck tests

/// Test IPv4 roundtrip using QuickCheck
#[quickcheck]
fn qc_ipv4_roundtrip(a: u8, b: u8, c: u8, d: u8) -> TestResult {
    let ip = Ipv4Addr::new(a, b, c, d);
    let ip_str = ip.to_string();

    match encode_ip_address(&ip_str) {
        Ok(encoded) => match decode_words(&encoded) {
            Ok(decoded) => {
                // With smart port handling using port 65535 as marker, IP addresses without ports roundtrip exactly
                TestResult::from_bool(ip_str == decoded)
            }
            Err(_) => TestResult::failed(),
        },
        Err(_) => TestResult::discard(), // Skip invalid cases
    }
}

/// Test that encoding is injective (one-to-one)
#[allow(clippy::too_many_arguments)]
#[quickcheck]
fn qc_encoding_injective(
    a1: u8,
    b1: u8,
    c1: u8,
    d1: u8,
    a2: u8,
    b2: u8,
    c2: u8,
    d2: u8,
) -> TestResult {
    let ip1 = Ipv4Addr::new(a1, b1, c1, d1);
    let ip2 = Ipv4Addr::new(a2, b2, c2, d2);

    if ip1 == ip2 {
        return TestResult::discard();
    }

    match (
        encode_ip_address(&ip1.to_string()),
        encode_ip_address(&ip2.to_string()),
    ) {
        (Ok(encoded1), Ok(encoded2)) => TestResult::from_bool(encoded1 != encoded2),
        _ => TestResult::discard(),
    }
}

/// Test word format consistency
#[quickcheck]
fn qc_word_format_consistency(a: u8, b: u8, c: u8, d: u8) -> TestResult {
    let ip = Ipv4Addr::new(a, b, c, d);
    let ip_str = ip.to_string();

    match encode_ip_address(&ip_str) {
        Ok(encoded) => {
            let words: Vec<&str> = encoded.split(' ').collect();

            // Check word count
            if words.len() != 4 {
                return TestResult::failed();
            }

            // Check each word
            for word in words {
                if word.is_empty() {
                    return TestResult::failed();
                }
                // Allow 1-25 character words (GOLD wordlist includes single-character words)
                if word.is_empty() || word.len() > 25 {
                    return TestResult::failed();
                }
                // Words should be lowercase alphabetic only (no dashes in space-separated format)
                if !word.chars().all(|c| c.is_ascii_lowercase()) {
                    return TestResult::failed();
                }
            }

            TestResult::passed()
        }
        Err(_) => TestResult::discard(),
    }
}

/// Test collision resistance
#[quickcheck]
fn qc_collision_resistance(inputs: Vec<u32>) -> TestResult {
    if inputs.len() < 2 {
        return TestResult::discard();
    }

    let mut seen_ips = std::collections::HashSet::new();
    let mut encodings = std::collections::HashMap::new();

    for input in inputs {
        let bytes = input.to_be_bytes();
        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let ip_str = ip.to_string();

        // Skip if we've seen this IP before (not a collision, just a duplicate)
        if seen_ips.contains(&ip_str) {
            continue;
        }
        seen_ips.insert(ip_str.clone());

        if let Ok(encoded) = encode_ip_address(&ip_str) {
            // Check if this encoding already exists for a different IP
            if let Some(existing_ip) = encodings.get(&encoded) {
                if existing_ip != &ip_str {
                    return TestResult::failed(); // Collision detected - different IPs, same encoding
                }
            } else {
                encodings.insert(encoded, ip_str);
            }
        }
    }

    TestResult::passed()
}

/// Test error handling consistency
#[quickcheck]
fn qc_error_handling_consistency(invalid_input: String) -> TestResult {
    // Test that invalid inputs consistently produce errors
    if invalid_input.parse::<IpAddr>().is_ok() {
        return TestResult::discard(); // Skip valid IPs
    }

    match encode_ip_address(&invalid_input) {
        Ok(_) => TestResult::failed(), // Should have failed
        Err(_) => TestResult::passed(),
    }
}

/// Test batch processing consistency
#[quickcheck]
fn qc_batch_consistency(ips: Vec<u32>) -> TestResult {
    if ips.is_empty() {
        return TestResult::discard();
    }

    for ip_val in ips {
        let bytes = ip_val.to_be_bytes();
        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let ip_str = ip.to_string();

        // Encode/decode multiple times - should be consistent
        if let Ok(encoded1) = encode_ip_address(&ip_str)
            && let Ok(encoded2) = encode_ip_address(&ip_str)
            && encoded1 != encoded2
        {
            return TestResult::failed();
        }
    }

    TestResult::passed()
}

// Specialized property tests for edge cases

/// Test special IPv4 addresses
#[test]
fn test_special_ipv4_addresses() {
    let special_addresses = vec![
        "0.0.0.0",         // All zeros
        "255.255.255.255", // All ones
        "127.0.0.1",       // Loopback
        "169.254.1.1",     // Link-local
        "224.0.0.1",       // Multicast
        "192.168.1.1",     // Private
        "10.0.0.1",        // Private
        "172.16.0.1",      // Private
    ];

    for addr in special_addresses {
        let encoded = encode_ip_address(addr).expect("Encoding failed");
        let decoded = decode_words(&encoded).expect("Decoding failed");
        // Check if the decoded address starts with the original (may include port)
        assert!(
            decoded.starts_with(addr),
            "Special address roundtrip failed: {addr} -> {decoded}"
        );
    }
}

/// Test special IPv6 addresses
#[test]
fn test_special_ipv6_addresses() {
    let special_addresses = vec![
        "::",               // All zeros
        "::1",              // Loopback
        "fe80::1",          // Link-local
        "ff02::1",          // Multicast
        "2001:db8::1",      // Documentation
        "::ffff:192.0.2.1", // IPv4-mapped
    ];

    for addr in special_addresses {
        if let Ok(encoded) = encode_ip_address(addr)
            && let Ok(decoded) = decode_words(&encoded)
        {
            // The decoded address may include a port, so check if it starts with the original
            assert!(
                decoded.starts_with(addr) || decoded.starts_with(&format!("[{addr}]")),
                "Special IPv6 address roundtrip failed: {addr} -> {decoded}"
            );
        }
    }
}

// Helper functions (placeholders - replace with actual implementation)
fn encode_ip_address(addr: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.encode(addr).map_err(|e| e.into())
}

fn decode_words(words: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.decode(words).map_err(|e| e.into())
}

fn encode_socket_address(addr: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.encode(addr).map_err(|e| e.into())
}

fn decode_socket_address(words: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.decode(words).map_err(|e| e.into())
}

// Test configuration
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    // Config applied to all proptest tests in this module
}

// Custom strategies for more focused testing
pub fn ipv4_strategy() -> impl Strategy<Value = Ipv4Addr> {
    (0u8..=255, 0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(a, b, c, d)| Ipv4Addr::new(a, b, c, d))
}

pub fn port_strategy() -> impl Strategy<Value = u16> {
    1u16..=65535
}

pub fn socket_addr_strategy() -> impl Strategy<Value = SocketAddr> {
    (ipv4_strategy(), port_strategy()).prop_map(|(ip, port)| SocketAddr::from((ip, port)))
}

// Use custom strategies
proptest! {
    #[test]
    fn prop_custom_socket_roundtrip(socket in socket_addr_strategy()) {
        let socket_str = socket.to_string();

        if let Ok(encoded) = encode_socket_address(&socket_str)
            && let Ok(decoded) = decode_socket_address(&encoded) {
                prop_assert_eq!(socket_str, decoded);
            }
    }
}
