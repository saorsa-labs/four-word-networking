use four_word_networking::{FourWordAdaptiveEncoder, FourWordIpv6Encoder};
use std::net::SocketAddrV6;

fn main() {
    let adaptive_encoder = FourWordAdaptiveEncoder::new().unwrap();
    let ipv6_encoder = FourWordIpv6Encoder::new();

    let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

    for addr_str in test_cases {
        println!("\n=== Debugging decode for: {addr_str} ===");

        // First, encode with direct IPv6 encoder to get the expected structure
        let addr: SocketAddrV6 = addr_str.parse().unwrap();
        let direct_encoded = ipv6_encoder.encode(&addr).unwrap();

        println!("Direct encoding:");
        println!("  Category: {:?}", direct_encoded.category());
        println!("  Groups: {}", direct_encoded.groups().len());
        for (i, group) in direct_encoded.groups().iter().enumerate() {
            println!("    Group {}: {:?}", i, group.words());
        }

        // Now try to decode this with the adaptive encoder
        let encoded_str = direct_encoded.to_string();
        println!("Encoded string: '{encoded_str}'");

        match adaptive_encoder.decode(&encoded_str) {
            Ok(decoded) => {
                println!("Adaptive decode success: {decoded}");
            }
            Err(e) => {
                println!("Adaptive decode error: {e}");

                // Try to understand what's happening in the parsing
                let word_count = encoded_str.split(' ').count();
                println!("Word count by split(' '): {word_count}");

                // Check what the parse_ipv6_groups would create
                // We can't access it directly, but we can infer the issue
            }
        }
    }
}
