use four_word_networking::{FourWordAdaptiveEncoder, FourWordIpv6Encoder};
use std::net::SocketAddr;

fn main() {
    println!("=== Debugging Google DNS encoding issue ===\n");

    let test_address = "[2001:4860:4860::8888]:53";
    println!("Test address: {test_address}");

    // Test with IPv6 encoder directly
    let ipv6_encoder = FourWordIpv6Encoder::new();
    let addr: SocketAddr = test_address.parse().unwrap();

    if let SocketAddr::V6(v6) = addr {
        println!("\nIPv6 address parsed: {v6}");

        // Encode
        match ipv6_encoder.encode(&v6) {
            Ok(encoded) => {
                println!("\nEncoded successfully:");
                println!("  Words: {encoded}");
                println!("  Word count: {}", encoded.word_count());
                println!("  Category: {:?}", encoded.category());
                println!("  Dashed format: {}", encoded.to_dashed_string());

                // Show the groups
                println!("\n  Groups ({}):", encoded.groups().len());
                for (i, group) in encoded.groups().iter().enumerate() {
                    println!("    Group {}: {:?}", i, group.words());
                }

                // Try to decode
                println!("\nAttempting decode...");
                match ipv6_encoder.decode(&encoded) {
                    Ok(decoded) => {
                        println!("  Decoded: {decoded}");
                        if decoded == v6 {
                            println!("  ✓ Roundtrip successful!");
                        } else {
                            println!("  ✗ Roundtrip FAILED!");
                            println!("    Expected IP: {}", v6.ip());
                            println!("    Got IP:      {}", decoded.ip());
                            println!("    Expected port: {}", v6.port());
                            println!("    Got port:      {}", decoded.port());
                        }
                    }
                    Err(e) => {
                        println!("  Decode error: {e:?}");
                    }
                }
            }
            Err(e) => {
                println!("Encode error: {e:?}");
            }
        }
    }

    // Test with adaptive encoder
    println!("\n=== Testing with adaptive encoder ===");
    let adaptive_encoder = FourWordAdaptiveEncoder::new().unwrap();

    match adaptive_encoder.encode(test_address) {
        Ok(encoded) => {
            println!("Encoded: '{encoded}'");
            match adaptive_encoder.decode(&encoded) {
                Ok(decoded) => {
                    println!("Decoded: '{decoded}'");
                    if decoded == test_address {
                        println!("✓ Adaptive roundtrip successful!");
                    } else {
                        println!("✗ Adaptive roundtrip FAILED!");
                    }
                }
                Err(e) => {
                    println!("Adaptive decode error: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("Adaptive encode error: {e:?}");
        }
    }
}
