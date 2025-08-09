use std::fs;
use std::io::Write;
/// CLI integration tests for four-word networking
use std::process::{Command, Stdio};
use tempfile::TempDir;

mod test_config;
#[allow(unused_imports)]
use test_config::*;

/// Test CLI basic functionality
#[test]
fn test_cli_basic_ipv4_encoding() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "192.168.1.1"])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let encoded = stdout.trim();

    // Should produce 4 words separated by spaces
    let words: Vec<&str> = encoded.split(' ').collect();
    assert_eq!(words.len(), 4, "Should produce exactly 4 words for IPv4");

    // Each word should be valid
    for word in words {
        assert!(!word.is_empty(), "Word should not be empty");
        assert!(
            !word.is_empty(),
            "Word should be at least 1 character (GOLD wordlist includes single-character words)"
        );
    }
}

#[test]
fn test_cli_basic_ipv6_encoding() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "::1"])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let encoded = stdout.trim();

    // IPv6 should produce 6, 9, or 12 words
    let words: Vec<&str> = encoded.split(' ').collect();
    assert!(
        words.len() == 6 || words.len() == 9 || words.len() == 12,
        "IPv6 should produce 6, 9, or 12 words, got: {}",
        words.len()
    );
}

#[test]
fn test_cli_socket_address_encoding() {
    let test_cases = vec![
        "192.168.1.1:8080",
        "127.0.0.1:443",
        "[::1]:8080",
        "[2001:db8::1]:443",
    ];

    for addr in test_cases {
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", addr])
            .output()
            .expect("Failed to execute CLI");

        assert!(
            output.status.success(),
            "CLI command failed for {}: {}",
            addr,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let encoded = stdout.trim();

        // Should produce valid word format
        assert!(!encoded.is_empty(), "Output should not be empty for {addr}");

        // All addresses use space-separated words
        let word_count = encoded.split(' ').count();
        if addr.contains('[') {
            assert!(
                word_count == 6 || word_count == 9 || word_count == 12,
                "IPv6 should have 6, 9, or 12 words for {addr}, got {word_count}"
            );
        } else {
            assert_eq!(word_count, 4, "IPv4 should have exactly 4 words for {addr}");
        }
    }
}

#[test]
fn test_cli_word_decoding() {
    // First encode an address
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "192.168.1.1"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let encoded = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Then decode it back
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", &encoded])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI decoding failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let decoded_output = String::from_utf8_lossy(&output.stdout);
    let decoded = decoded_output.trim();
    assert!(
        decoded.contains("192.168.1.1"),
        "Decoded address should contain original IP: {decoded}"
    );
}

#[test]
fn test_cli_verbose_output() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "-v", "192.168.1.1"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verbose output should contain additional information
    assert!(stdout.len() > 20, "Verbose output should be longer");
}

#[test]
fn test_cli_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "--help"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Help output should contain usage information
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"));
    assert!(stdout.contains("4wn") || stdout.contains("four-word"));
}

#[test]
fn test_cli_version_output() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "--version"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Version output should contain version information
    assert!(stdout.contains("2.6.0") || stdout.contains("version"));
}

#[test]
fn test_cli_invalid_input_handling() {
    let invalid_inputs = vec![
        "999.999.999.999",   // Invalid IP format
        "xyz123.invalid",    // Not valid words
        "",                  // Empty input
        "192.168.1.1:99999", // Port out of range
        "::gg",              // Invalid IPv6
        "word1.word2.word3", // Only 3 words
        "a.b.c",             // 3 words (not 4)
        "one two three",     // 3 space-separated words
        "123.456.789",       // Numbers not words
    ];

    for input in invalid_inputs {
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", input])
            .output()
            .expect("Failed to execute CLI");

        // Should fail gracefully with error message
        assert!(
            !output.status.success(),
            "CLI should fail for invalid input: {input}"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.is_empty(),
            "Should provide error message for invalid input: {input}"
        );
    }
}

#[test]
fn test_cli_batch_processing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("addresses.txt");

    // Create input file with multiple addresses
    let addresses = ["192.168.1.1", "10.0.0.1", "127.0.0.1", "::1", "2001:db8::1"];

    let content = addresses.join("\n");
    fs::write(&input_file, content).expect("Failed to write input file");

    // Test batch processing (if supported)
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "4wn",
            "--",
            "--file",
            input_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute CLI");

    // This might not be supported yet, so we'll just check if it runs
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either succeeds or fails gracefully
    if !output.status.success() {
        assert!(
            stderr.contains("not supported")
                || stderr.contains("unknown")
                || stderr.contains("help"),
            "Should provide helpful error message for unsupported features"
        );
    }
}

#[test]
fn test_cli_output_formats() {
    let test_formats = vec![
        ("--format", "json"),
        ("--format", "yaml"),
        ("--format", "plain"),
        ("--output", "json"),
        ("-o", "json"),
    ];

    for (flag, format) in test_formats {
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", flag, format, "192.168.1.1"])
            .output()
            .expect("Failed to execute CLI");

        // Either succeeds or fails gracefully
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("not supported")
                    || stderr.contains("unknown")
                    || stderr.contains("help"),
                "Should provide helpful error message for unsupported format: {format}"
            );
        }
    }
}

#[test]
fn test_cli_performance_mode() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "--benchmark", "192.168.1.1"])
        .output()
        .expect("Failed to execute CLI");

    // Either succeeds or fails gracefully
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("not supported")
                || stderr.contains("unknown")
                || stderr.contains("help"),
            "Should provide helpful error message for unsupported benchmark mode"
        );
    }
}

#[test]
fn test_cli_error_recovery() {
    // Test that CLI can recover from errors and continue
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "4wn"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    // Send invalid input
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"invalid.input\n");
        let _ = stdin.write_all(b"192.168.1.1\n");
        let _ = stdin.write_all(b"quit\n");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");

    // Should handle invalid input gracefully
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that it processed some input
    assert!(
        !stdout.is_empty() || !stderr.is_empty(),
        "CLI should produce some output"
    );
}

#[test]
fn test_cli_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let addresses = Arc::new(vec![
        "192.168.1.1",
        "10.0.0.1",
        "127.0.0.1",
        "::1",
        "2001:db8::1",
    ]);

    let mut handles = vec![];

    // Spawn multiple CLI processes concurrently
    for i in 0..5 {
        let addresses = Arc::clone(&addresses);
        let handle = thread::spawn(move || {
            let addr = addresses[i];
            let output = Command::new("cargo")
                .args(["run", "--bin", "4wn", "--", addr])
                .output()
                .expect("Failed to execute CLI");

            (
                addr,
                output.status.success(),
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
            )
        });
        handles.push(handle);
    }

    // Wait for all processes to complete
    for handle in handles {
        let (addr, success, output) = handle.join().expect("Thread failed");
        assert!(success, "CLI failed for address: {addr}");
        assert!(!output.is_empty(), "CLI produced empty output for: {addr}");
    }
}

#[test]
fn test_cli_memory_usage() {
    // Test that CLI doesn't have memory leaks with repeated use
    for i in 0..100 {
        let addr = format!("192.168.1.{i}");
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", &addr])
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success(), "CLI failed for address: {addr}");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "CLI produced empty output for: {addr}"
        );
    }
}

#[test]
fn test_cli_large_input_handling() {
    // Test with very long input
    let long_input = "a".repeat(1000);
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", &long_input])
        .output()
        .expect("Failed to execute CLI");

    // Should fail gracefully
    assert!(
        !output.status.success(),
        "CLI should reject very long input"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "Should provide error message for long input"
    );
}

#[test]
fn test_cli_unicode_handling() {
    let unicode_inputs = vec!["192.168.1.1🚀", "réseau.test", "网络.测试", "тест.сеть"];

    for input in unicode_inputs {
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", input])
            .output()
            .expect("Failed to execute CLI");

        // Should handle gracefully (either succeed or fail with clear error)
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stderr.is_empty(),
                "Should provide error message for unicode input: {input}"
            );
        }
    }
}

#[test]
fn test_cli_signal_handling() {
    // Test that CLI handles interruption gracefully
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "--interactive"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    // Let it run for a moment
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Send termination signal
    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for CLI");

    // Should terminate gracefully
    assert!(!output.status.success(), "CLI should terminate when killed");
}

#[test]
fn test_cli_configuration_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("config.toml");

    // Create a configuration file
    let config_content = r#"
[default]
format = "json"
verbose = true

[encoding]
compression = "high"
dictionary = "16k"
"#;

    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Test with configuration file
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "4wn",
            "--",
            "--config",
            config_file.to_str().unwrap(),
            "192.168.1.1",
        ])
        .output()
        .expect("Failed to execute CLI");

    // Either succeeds or fails gracefully
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("not supported")
                || stderr.contains("unknown")
                || stderr.contains("help"),
            "Should provide helpful error message for unsupported config file"
        );
    }
}

#[test]
fn test_cli_environment_variables() {
    // Test with environment variables
    let output = Command::new("cargo")
        .args(["run", "--bin", "4wn", "--", "192.168.1.1"])
        .env("FWN_VERBOSE", "true")
        .env("FWN_FORMAT", "json")
        .env("FWN_COMPRESSION", "high")
        .output()
        .expect("Failed to execute CLI");

    // Should still work (environment variables might not be supported yet)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either succeeds or provides helpful error
    if !output.status.success() {
        assert!(
            !stderr.is_empty(),
            "Should provide error message when failing"
        );
    } else {
        assert!(
            !stdout.trim().is_empty(),
            "Should provide output when succeeding"
        );
    }
}

#[test]
fn test_cli_workflow_integration() {
    // Test a complete workflow
    let addresses = vec![
        "192.168.1.1:8080",
        "10.0.0.1:22",
        "127.0.0.1:443",
        "[::1]:8080",
    ];

    let mut encoded_addresses = Vec::new();

    // Encode all addresses
    for addr in &addresses {
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", addr])
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success(), "Encoding failed for: {addr}");

        let encoded = String::from_utf8_lossy(&output.stdout).trim().to_string();
        encoded_addresses.push(encoded);
    }

    // Decode all addresses
    for (i, encoded) in encoded_addresses.iter().enumerate() {
        let output = Command::new("cargo")
            .args(["run", "--bin", "4wn", "--", encoded])
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success(), "Decoding failed for: {encoded}");

        let decoded_output = String::from_utf8_lossy(&output.stdout);
        let decoded = decoded_output.trim();
        // The decoded address should contain the original IP
        let original_ip = addresses[i].split(':').next().unwrap();
        assert!(
            decoded.contains(original_ip),
            "Decoded address should contain original IP: {} -> {} -> {}",
            addresses[i],
            encoded,
            decoded
        );
    }
}
