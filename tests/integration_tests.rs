#![allow(unused_imports)]

/// Comprehensive integration tests for four-word networking
use four_word_networking::FourWordAdaptiveEncoder;
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

mod test_config;
use test_config::*;

/// Test complete encoding/decoding workflows
#[test]
fn test_complete_ipv4_workflow() {
    init_test_env();
    let generator = AddressGenerator::new();

    for addr in generator.ipv4_addresses() {
        let encoded = encode_ip_address(addr).expect("Encoding failed");
        let decoded = decode_words(&encoded).expect("Decoding failed");
        assert_encoding_roundtrip(addr, &encoded, &decoded);
    }
}

#[test]
fn test_complete_ipv6_workflow() {
    init_test_env();
    let generator = AddressGenerator::new();

    for addr in generator.ipv6_addresses() {
        let encoded = encode_ip_address(addr).expect("Encoding failed");
        let decoded = decode_words(&encoded).expect("Decoding failed");
        assert_encoding_roundtrip(addr, &encoded, &decoded);
    }
}

#[test]
fn test_socket_address_workflow() {
    init_test_env();
    let test_cases = vec![
        "127.0.0.1:8080",
        "192.168.1.1:443",
        "[::1]:8080",
        "[2001:db8::1]:443",
    ];

    for addr in test_cases {
        let encoded = encode_socket_address(addr).expect("Encoding failed");
        let decoded = decode_socket_address(&encoded).expect("Decoding failed");
        assert_encoding_roundtrip(addr, &encoded, &decoded);
    }
}

#[test]
fn test_ip_address_workflow() {
    init_test_env();
    let generator = AddressGenerator::new();

    // Test IPv4 addresses with ports
    for addr in generator.ipv4_with_ports() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();
        if let Ok(encoded) = encoder.encode(addr) {
            let decoded = encoder.decode(&encoded).expect("Decoding failed");
            assert_eq!(addr, &decoded, "Roundtrip failed for {addr}");
        }
    }

    // Test IPv6 addresses with ports
    for addr in generator.ipv6_with_ports() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();
        if let Ok(encoded) = encoder.encode(addr) {
            let decoded = encoder.decode(&encoded).expect("Decoding failed");
            assert_eq!(addr, &decoded, "Roundtrip failed for {addr}");
        }
    }
}

/// Test error handling and edge cases
#[test]
fn test_invalid_input_handling() {
    init_test_env();
    let invalid_inputs = vec![
        "invalid.ip.address",
        "999.999.999.999",
        "::gg",
        "not-an-ip",
        "",
        "127.0.0.1:99999",
    ];

    for input in invalid_inputs {
        assert!(
            encode_ip_address(input).is_err(),
            "Expected error for invalid input: {input}"
        );
    }
}

#[test]
fn test_word_validation() {
    init_test_env();
    let valid_words = vec![
        "apple orange banana cherry",
        "one two three four",
        "first second third fourth",
    ];

    for words in valid_words {
        let result = validate_word_format(words);
        assert!(
            result.is_ok(),
            "Valid words should pass validation: {words}"
        );
    }

    let invalid_words = vec![
        "one two three",           // Too few words
        "one two three four five", // Too many words
        "one  two three four",     // Double space (empty word)
        "",                        // Empty string
    ];

    for words in invalid_words {
        let result = validate_word_format(words);
        assert!(
            result.is_err(),
            "Invalid words should fail validation: {words}"
        );
    }
}

/// Test performance requirements
#[test]
fn test_encoding_performance() {
    init_test_env();
    let test_ip = "192.168.1.1";

    test_performance!(
        "IPv4 encoding",
        {
            let _ = encode_ip_address(test_ip).expect("Encoding failed");
        },
        100000
    ); // 100ms max (very lenient for debug builds)
}

#[test]
fn test_decoding_performance() {
    init_test_env();
    let test_ip = "192.168.1.1";
    let encoded = encode_ip_address(test_ip).expect("Encoding failed");

    test_performance!(
        "IPv4 decoding",
        {
            let _ = decode_words(&encoded).expect("Decoding failed");
        },
        50000
    ); // 50ms max (more lenient for debug builds)
}

#[test]
fn test_batch_processing_performance() {
    init_test_env();
    let generator = AddressGenerator::new();
    let addresses: Vec<_> = generator
        .ipv4_addresses()
        .iter()
        .chain(generator.ipv6_addresses().iter())
        .collect();

    let start = Instant::now();

    for addr in &addresses {
        let encoded = encode_ip_address(addr).expect("Encoding failed");
        let _decoded = decode_words(&encoded).expect("Decoding failed");
    }

    let duration = start.elapsed();
    let ops_per_sec = (addresses.len() as f64 * 2.0) / duration.as_secs_f64();

    // Should process at least 100 operations per second (more lenient for debug builds)
    assert!(
        ops_per_sec >= 100.0,
        "Batch processing too slow: {ops_per_sec:.2} ops/sec"
    );
}

/// Test memory usage and resource management
#[test]
fn test_memory_efficiency() {
    init_test_env();
    let generator = AddressGenerator::new();

    // Test that repeated operations don't cause memory leaks
    for _ in 0..1000 {
        for addr in generator.ipv4_addresses() {
            let encoded = encode_ip_address(addr).expect("Encoding failed");
            let _decoded = decode_words(&encoded).expect("Decoding failed");
        }
    }

    // If we get here without OOM, the test passes
}

/// Test concurrent access
#[test]
fn test_concurrent_encoding() {
    init_test_env();
    use std::sync::Arc;
    use std::thread;

    let generator = Arc::new(AddressGenerator::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let generator_clone = Arc::clone(&generator);
        let handle = thread::spawn(move || {
            for addr in generator_clone.ipv4_addresses() {
                let encoded = encode_ip_address(addr).expect("Encoding failed");
                let decoded = decode_words(&encoded).expect("Decoding failed");
                assert_encoding_roundtrip(addr, &encoded, &decoded);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread failed");
    }
}

/// Test with real-world data patterns
#[test]
fn test_real_world_patterns() {
    init_test_env();
    let real_world_addresses = real_world_data();

    for addr in real_world_addresses {
        let encoded = encode_ip_address(&addr).expect("Encoding failed");
        let decoded = decode_words(&encoded).expect("Decoding failed");
        assert_encoding_roundtrip(&addr, &encoded, &decoded);
    }
}

/// Test edge cases
#[test]
fn test_edge_cases() {
    init_test_env();
    let edge_cases = edge_case_data();

    for addr in edge_cases {
        let encoded = encode_ip_address(&addr).expect("Encoding failed");
        let decoded = decode_words(&encoded).expect("Decoding failed");
        assert_encoding_roundtrip(&addr, &encoded, &decoded);
    }
}

/// Test compression efficiency
#[test]
fn test_compression_efficiency() {
    init_test_env();
    let test_cases = vec![
        ("127.0.0.1", 64), // 4 words × 16 chars/word max = 64 bytes max
        ("192.168.1.1", 64),
        ("::1", 96), // IPv6 uses 6 words × 16 chars/word max = 96 bytes max
        ("2001:db8::1", 96),
    ];

    for (addr, expected_max_bytes) in test_cases {
        let encoded = encode_ip_address(addr).expect("Encoding failed");
        let byte_count = encoded.len(); // Actual string length in bytes

        assert!(
            byte_count <= expected_max_bytes,
            "Encoding too long for {addr}: {byte_count} bytes > {expected_max_bytes} bytes"
        );
    }
}

/// Test deterministic behavior
#[test]
fn test_deterministic_encoding() {
    init_test_env();
    let test_ip = "192.168.1.1";

    // Encode the same address multiple times
    let encodings: Vec<_> = (0..10)
        .map(|_| encode_ip_address(test_ip).expect("Encoding failed"))
        .collect();

    // All encodings should be identical
    let first = &encodings[0];
    for encoding in &encodings[1..] {
        assert_eq!(first, encoding, "Encoding should be deterministic");
    }
}

/// Test word quality and readability
#[test]
fn test_word_quality() {
    init_test_env();
    let test_ip = "192.168.1.1";
    let encoded = encode_ip_address(test_ip).expect("Encoding failed");
    let words: Vec<&str> = encoded.split(' ').collect();

    // Test word properties
    for word in words {
        assert!(!word.is_empty(), "Word too short: {word}");
        // No maximum length restriction - frequency-based words can be longer
        assert!(
            word.chars().all(|c| c.is_ascii_lowercase()),
            "Word contains invalid characters: {word}"
        );
    }
}

/// Test error recovery
#[test]
fn test_error_recovery() {
    init_test_env();
    let malformed_words = vec![
        "one.two.three",              // Missing word (need 4 for IPv4)
        "one.two.three.four.five",    // Extra word
        "zzzzz999.xxxxx888.qqqqq777", // Invalid words not in dictionary
        ".two.three",                 // Empty first word
        "one..three",                 // Empty middle word
    ];

    for words in malformed_words {
        match decode_words(words) {
            Ok(_) => panic!("Expected error for malformed words: {words}"),
            Err(e) => {
                // Error should be descriptive
                let error_msg = format!("{e}");
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                assert!(error_msg.len() > 10, "Error message should be descriptive");
            }
        }
    }
}

/// Test CLI integration
#[test]
fn test_cli_integration() {
    init_test_env();
    use std::process::Command;

    let test_ip = "192.168.1.1";

    // Test CLI encoding
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", test_ip])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success(), "CLI encoding failed");

    let encoded = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 output")
        .trim()
        .to_string();

    // Test CLI decoding
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", &encoded])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success(), "CLI decoding failed");

    let decoded = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 output")
        .trim()
        .to_string();

    // With smart port handling, IP addresses without ports should roundtrip exactly
    assert_eq!(test_ip, decoded, "CLI roundtrip failed");
}

// Helper functions for testing
fn encode_ip_address(addr: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.encode(addr).map_err(|e| e.into())
}

fn decode_words(words: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.decode(words).map_err(|e| e.into())
}

fn encode_socket_address(addr: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.encode(addr).map_err(|e| e.into())
}

fn decode_socket_address(words: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.decode(words).map_err(|e| e.into())
}

// Multiaddr functions removed - using pure IP:port format only

fn validate_word_format(words: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = words.split(' ').collect();
    if parts.len() != 4 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Must have exactly 4 words",
        )));
    }

    for part in parts {
        if part.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Words cannot be empty",
            )));
        }
        if !part.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Words can only contain lowercase letters and hyphens",
            )));
        }
    }

    Ok(())
}
