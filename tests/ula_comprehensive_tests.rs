//! Comprehensive tests for Unique Local Addresses (ULA) fc00::/7
//! This ensures the fc00:: duplication bug is fully fixed and doesn't regress

use four_word_networking::FourWordAdaptiveEncoder;
use std::collections::HashSet;

#[test]
fn test_ula_basic_addresses() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test basic ULA addresses
    let test_cases = vec![
        ("[fc00::]:443", "Basic fc00::"),
        ("[fc01::]:443", "fc01::"),
        ("[fc02::]:443", "fc02::"),
        ("[fcff::]:443", "fcff::"),
        ("[fd00::]:443", "Basic fd00::"),
        ("[fd01::]:443", "fd01::"),
        ("[fd02::]:443", "fd02::"),
        ("[fdff::]:443", "fdff::"),
    ];

    for (addr, description) in test_cases {
        println!("Testing {description}: {addr}");

        let encoded = encoder.encode(addr).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded, addr, "Failed for {description}");

        // Ensure no duplication in decoded address
        assert!(
            !decoded.contains("fc00:fc00"),
            "Duplication bug detected for {description}"
        );
        assert!(
            !decoded.contains("fd00:fd00"),
            "Duplication bug detected for {description}"
        );
    }
}

#[test]
fn test_ula_with_global_id() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test ULA addresses with various global IDs
    let test_cases = vec![
        "[fc00:1234:5678::]:443",
        "[fc00:abcd:ef01::]:443",
        "[fd00:1234:5678::]:443",
        "[fd00:abcd:ef01::]:443",
        "[fc12:3456:7890::]:443",
        "[fd12:3456:7890::]:443",
    ];

    for addr in test_cases {
        println!("Testing ULA with global ID: {addr}");

        let encoded = encoder.encode(addr).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded, addr, "Failed for {addr}");
    }
}

#[test]
fn test_ula_with_subnet_id() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test ULA addresses with subnet IDs
    // Note: Compression only preserves first 4 segments (64 bits)
    // So segments[4-7] will be lost during compression
    let test_cases = vec![
        ("[fc00:1234:5678:9abc::]:443", "[fc00:1234:5678:9abc::]:443"),
        (
            "[fc00:1234:5678:9abc:def0::]:443",
            "[fc00:1234:5678:9abc::]:443",
        ), // def0 is segment[4], will be lost
        ("[fd00:1234:5678:9abc::]:443", "[fd00:1234:5678:9abc::]:443"),
        (
            "[fd00:1234:5678:9abc:def0::]:443",
            "[fd00:1234:5678:9abc::]:443",
        ), // def0 is segment[4], will be lost
        ("[fc00:0:0:1234::]:443", "[fc00:0:0:1234::]:443"),
        ("[fd00:0:0:5678::]:443", "[fd00:0:0:5678::]:443"),
    ];

    for (addr, expected) in test_cases {
        println!("Testing ULA with subnet ID: {addr} -> {expected}");

        let encoded = encoder.encode(addr).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded, expected, "Failed for {addr}");
    }
}

#[test]
fn test_ula_with_interface_id() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test ULA addresses with interface IDs (these should lose the interface ID)
    let test_cases = vec![
        ("[fc00::1]:443", "[fc00::]:443"),
        ("[fc00::2]:443", "[fc00::]:443"),
        ("[fc00::ffff]:443", "[fc00::]:443"),
        ("[fd00::1]:443", "[fd00::]:443"),
        ("[fd00::2]:443", "[fd00::]:443"),
        ("[fd00::ffff]:443", "[fd00::]:443"),
        (
            "[fc00:1234:5678:9abc::1]:443",
            "[fc00:1234:5678:9abc::]:443",
        ),
        (
            "[fd00:1234:5678:9abc::1]:443",
            "[fd00:1234:5678:9abc::]:443",
        ),
    ];

    for (input, expected) in test_cases {
        println!("Testing ULA with interface ID: {input} -> {expected}");

        let encoded = encoder.encode(input).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded, expected, "Failed for {input}");
    }
}

#[test]
fn test_ula_encoding_uniqueness() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Ensure different ULA addresses produce different encodings
    let addresses = vec![
        "[fc00::]:443",
        "[fc01::]:443",
        "[fc02::]:443",
        "[fd00::]:443",
        "[fd01::]:443",
        "[fd02::]:443",
        "[fc00:1234::]:443",
        "[fc00:5678::]:443",
        "[fd00:1234::]:443",
        "[fd00:5678::]:443",
    ];

    let mut encodings = HashSet::new();

    for addr in addresses {
        let encoded = encoder.encode(addr).expect("Failed to encode");

        // Check that this encoding is unique
        assert!(
            encodings.insert(encoded.clone()),
            "Duplicate encoding found for {addr}: {encoded}"
        );
    }
}

#[test]
fn test_ula_word_count() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // All ULA addresses should use 6 words (compressed category)
    let addresses = vec![
        "[fc00::]:443",
        "[fd00::]:443",
        "[fc00:1234:5678:9abc::]:443",
        "[fd00:1234:5678:9abc::]:443",
    ];

    for addr in addresses {
        let encoded = encoder.encode(addr).expect("Failed to encode");
        let word_count = encoded.split(' ').count();

        assert_eq!(
            word_count, 6,
            "ULA address {addr} should encode to 6 words, got {word_count}"
        );
    }
}

#[test]
fn test_ula_edge_cases() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test edge cases and boundary values
    let test_cases = vec![
        // Minimum and maximum fc/fd values
        ("[fc00::]:443", "Minimum fc00"),
        ("[fcff:ffff:ffff:ffff::]:443", "Maximum fcff"),
        ("[fd00::]:443", "Minimum fd00"),
        ("[fdff:ffff:ffff:ffff::]:443", "Maximum fdff"),
        // Special patterns - note that IPv6 uses compressed format
        ("[fc00:0:0:1::]:443", "fc00 with minimal subnet"),
        ("[fd00:0:0:1::]:443", "fd00 with minimal subnet"),
        ("[fc00:ffff:ffff:ffff::]:443", "fc00 with maximum subnet"),
        ("[fd00:ffff:ffff:ffff::]:443", "fd00 with maximum subnet"),
    ];

    for (addr, description) in test_cases {
        println!("Testing edge case {description}: {addr}");

        let encoded = encoder.encode(addr).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded, addr, "Failed for {description}");

        // Extra check: ensure the prefix is preserved correctly
        assert!(
            decoded.starts_with("[fc") || decoded.starts_with("[fd"),
            "ULA prefix not preserved for {description}"
        );
    }
}

#[test]
fn test_ula_with_different_ports() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test that different ports produce different encodings
    let base_addr = "fc00::";
    let ports = vec![80, 443, 8080, 22, 3389, 65535];
    let mut encodings = HashSet::new();

    for port in ports {
        let addr = format!("[{base_addr}]:{port}");
        let encoded = encoder.encode(&addr).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded, addr, "Failed for port {port}");

        // Ensure unique encoding for each port
        assert!(
            encodings.insert(encoded.clone()),
            "Duplicate encoding found for port {port}: {encoded}"
        );
    }
}

#[test]
fn test_no_regression_fc00_duplication() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Specific regression test for the fc00:fc00:: duplication bug
    let critical_addresses = vec![
        "[fc00::]:443",
        "[fc00::1]:443",
        "[fc01::]:443",
        "[fd00::]:443",
        "[fd01::]:443",
    ];

    for addr in critical_addresses {
        let encoded = encoder.encode(addr).expect("Failed to encode");
        let decoded = encoder.decode(&encoded).expect("Failed to decode");

        // The critical check: no duplication patterns
        assert!(
            !decoded.contains("fc00:fc00"),
            "REGRESSION: fc00 duplication bug reappeared for {addr}"
        );
        assert!(
            !decoded.contains("fd00:fd00"),
            "REGRESSION: fd00 duplication bug reappeared for {addr}"
        );
        assert!(
            !decoded.contains("fc01:fc01"),
            "REGRESSION: fc01 duplication bug for {addr}"
        );
        assert!(
            !decoded.contains("fd01:fd01"),
            "REGRESSION: fd01 duplication bug for {addr}"
        );
    }
}
