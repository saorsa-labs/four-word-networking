use four_word_networking::{FourWordAdaptiveEncoder, FourWordIpv6Encoder};
use std::net::SocketAddrV6;

fn main() {
    let adaptive_encoder = FourWordAdaptiveEncoder::new().unwrap();
    let ipv6_encoder = FourWordIpv6Encoder::new();

    let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

    for addr_str in test_cases {
        println!("\n=== Comparing encoders for: {addr_str} ===");

        // Test direct IPv6 encoder
        let addr: SocketAddrV6 = addr_str.parse().unwrap();
        match ipv6_encoder.encode(&addr) {
            Ok(direct_encoded) => {
                println!("Direct IPv6 encoder:");
                println!("  Encoded: '{direct_encoded}'");
                println!("  Word count: {}", direct_encoded.word_count());
                println!("  Groups: {:?}", direct_encoded.groups().len());
                for (i, group) in direct_encoded.groups().iter().enumerate() {
                    println!("    Group {}: {:?}", i, group.words());
                }

                match ipv6_encoder.decode(&direct_encoded) {
                    Ok(decoded) => {
                        println!("  Decoded: {}:{}", decoded.ip(), decoded.port());
                    }
                    Err(e) => {
                        println!("  Decode error: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Direct IPv6 encoder error: {e}");
            }
        }

        // Test adaptive encoder
        match adaptive_encoder.encode(addr_str) {
            Ok(adaptive_encoded) => {
                println!("Adaptive encoder:");
                println!("  Encoded: '{adaptive_encoded}'");
                let word_count = adaptive_encoded.split_whitespace().count();
                println!("  Word count: {word_count}");

                match adaptive_encoder.decode(&adaptive_encoded) {
                    Ok(decoded) => {
                        println!("  Decoded: '{decoded}'");
                    }
                    Err(e) => {
                        println!("  Decode error: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Adaptive encoder error: {e}");
            }
        }
    }
}
