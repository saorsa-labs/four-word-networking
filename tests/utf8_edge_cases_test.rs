//! Tests for UTF-8 edge cases in address parsing

use four_word_networking::FourWordAdaptiveEncoder;

#[test]
fn test_utf8_characters_after_bracket() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test various UTF-8 characters after closing bracket
    let test_cases = vec![
        ("[::1]🚀:443", "Emoji after bracket"),
        ("[::1]é:443", "Accented character after bracket"),
        ("[::1]中:443", "Chinese character after bracket"),
        ("[::1]ñ:443", "Spanish ñ after bracket"),
        ("[::1]🎯:443", "Target emoji after bracket"),
        ("[2001:db8::1]🌍:80", "Globe emoji after bracket"),
    ];

    for (input, description) in test_cases {
        // Should not panic and should handle gracefully
        match encoder.encode(input) {
            Ok(encoded) => {
                // If it encodes successfully, it should decode back to a valid address
                let decoded = encoder.decode(&encoded).expect("Failed to decode");
                assert!(
                    decoded.starts_with('['),
                    "Decoded should be IPv6 format for {description}"
                );
                assert!(
                    decoded.contains("]:"),
                    "Decoded should contain port separator for {description}"
                );
            }
            Err(_) => {
                // It's also acceptable to return an error for malformed input
            }
        }
    }
}

#[test]
fn test_no_panic_on_utf8_boundaries() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // These should never panic, even if they return errors
    let edge_cases = vec![
        "[::1]",         // No port
        "[::1]:",        // Colon but no port
        "[::1]:🚀",      // Invalid port (emoji)
        "[::1]🚀",       // UTF-8 character, no colon
        "[🚀::1]:443",   // Invalid: UTF-8 in address
        "[:café:1]:443", // Invalid: UTF-8 in address
        "[",             // Incomplete
        "[::1",          // Missing closing bracket
        "[::1]]]:443",   // Multiple brackets
        "[::1]]:443",    // Extra bracket
    ];

    for input in edge_cases {
        // The important thing is that these don't panic
        let _ = encoder.encode(input);
    }
}

#[test]
fn test_valid_ipv6_with_port_after_utf8_fix() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // These should work correctly
    let valid_cases = vec![
        ("[::1]:443", "[::1]:443"),
        ("[2001:db8::1]:80", "[2001:db8::1]:80"),
        ("[fe80::1]:22", "[fe80::1]:22"), // Link-local preserves interface ID
        ("[fc00::1]:8080", "[fc00::]:8080"), // ULA loses interface ID
    ];

    for (input, expected_prefix) in valid_cases {
        let encoded = encoder
            .encode(input)
            .expect("Should encode valid IPv6 with port");
        let decoded = encoder.decode(&encoded).expect("Should decode back");

        assert!(
            decoded.starts_with(expected_prefix) || decoded == expected_prefix,
            "Expected {input} to decode to something starting with {expected_prefix}, got {decoded}"
        );
    }
}

#[test]
fn test_invalid_utf8_in_address() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // These should fail because IPv6 addresses can't contain UTF-8
    let invalid_addresses = vec![
        "[2001:db8::café]:443", // Accented character in address
        "[2001:db8::🚀]:443",   // Emoji in address
        "[fe80::中文]:22",      // Chinese characters in address
    ];

    for addr in invalid_addresses {
        assert!(
            encoder.encode(addr).is_err(),
            "Should reject IPv6 address with UTF-8 characters: {addr}"
        );
    }
}

#[test]
fn test_multibyte_port_separator() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Test various separators between address and port
    let test_cases = vec![
        ("[::1]：443", "Full-width colon"),     // U+FF1A
        ("[::1]꞉443", "Modifier letter colon"), // U+A789
        ("[::1]∶443", "Ratio symbol"),          // U+2236
    ];

    for (input, description) in test_cases {
        // These should not be recognized as valid port separators
        match encoder.encode(input) {
            Ok(encoded) => {
                let decoded = encoder.decode(&encoded).expect("Failed to decode");
                // Should decode with default port or port 0
                assert!(
                    decoded.ends_with(":0") || !decoded.contains(':'),
                    "Should not parse port with non-ASCII separator for {description}"
                );
            }
            Err(_) => {
                // Also acceptable to reject
            }
        }
    }
}

#[test]
fn test_string_slicing_safety() {
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

    // Create strings where slicing could be problematic
    let mut tricky_strings = vec![
        "192.168.1.1:🚀".to_string(),    // UTF-8 after port separator
        "192.168.1.1🚀:443".to_string(), // UTF-8 before port separator
        "192.168.🚀.1:443".to_string(),  // UTF-8 in IP (invalid)
        "[::🚀]:443".to_string(),        // UTF-8 in IPv6 (invalid)
    ];

    // Add a string with many UTF-8 characters
    let mut many_emojis = "[::1]".to_string();
    many_emojis.push_str(&"🚀".repeat(100));
    tricky_strings.push(many_emojis);

    for input in &tricky_strings {
        // Should handle without panicking
        let _ = encoder.encode(input);
    }
}
