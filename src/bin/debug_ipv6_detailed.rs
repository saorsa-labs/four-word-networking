use four_word_networking::FourWordAdaptiveEncoder;
use std::net::Ipv6Addr;

fn main() {
    println!("=== Detailed IPv6 encoding/decoding debug ===\n");

    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    // Test specific problematic addresses
    let test_addresses = vec![
        ("::1", "Loopback"),
        ("fe80::1", "Link-local"),
        ("::", "Unspecified"),
        ("2001:db8::1", "Documentation"),
    ];

    for (addr, description) in test_addresses {
        println!("\n========================================");
        println!("Testing: {addr} ({description})");
        println!("========================================");

        match encoder.encode(addr) {
            Ok(encoded) => {
                println!("Encoded: '{encoded}'");
                let word_count = encoded.split_whitespace().count();
                println!("Word count: {word_count}");

                // Try to decode with detailed error handling
                match encoder.decode(&encoded) {
                    Ok(decoded) => {
                        println!("Decoded: '{decoded}'");

                        // Check if it matches
                        if decoded == addr {
                            println!("✓ Exact match!");
                        } else if decoded == format!("[{addr}]:0") {
                            println!("✓ Match with added port (expected)");
                        } else {
                            println!("✗ MISMATCH!");
                            println!("  Expected: {addr}");
                            println!("  Got: {decoded}");
                        }
                    }
                    Err(e) => {
                        println!("Decode error: {e:?}");

                        // Try to understand the error
                        match e {
                            four_word_networking::FourWordError::InvalidInput(msg) => {
                                println!("  Error detail: {msg}");
                            }
                            _ => {
                                println!("  Other error type");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Encode error: {e:?}");
            }
        }
    }

    println!("\n\n=== Testing category detection ===");

    // Test category detection for each address type
    let test_ips = vec![
        (Ipv6Addr::LOCALHOST, "Loopback"),
        (Ipv6Addr::UNSPECIFIED, "Unspecified"),
        ("fe80::1".parse::<Ipv6Addr>().unwrap(), "Link-local"),
        ("2001:db8::1".parse::<Ipv6Addr>().unwrap(), "Documentation"),
        (
            "2001:4860:4860::8888".parse::<Ipv6Addr>().unwrap(),
            "Global unicast",
        ),
    ];

    use four_word_networking::ipv6_compression::Ipv6Compressor;
    let compressor = Ipv6Compressor::new();

    for (ip, expected) in test_ips {
        println!("\nIP: {ip} (expected: {expected})");
        match compressor.compress(ip, Some(0)) {
            Ok(compressed) => {
                println!("  Category: {:?}", compressed.category);
                println!(
                    "  Compressed size: {} bytes",
                    compressed.compressed_data.len()
                );
                println!("  Category bits: {}", compressed.category.to_bits());
            }
            Err(e) => {
                println!("  Compression error: {e:?}");
            }
        }
    }
}
