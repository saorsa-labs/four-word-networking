use four_word_networking::FourWordIpv6Encoder;
use std::net::SocketAddrV6;

fn main() {
    let encoder = FourWordIpv6Encoder::new();
    let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

    for addr_str in test_cases {
        println!("\n=== Analyzing string for: {addr_str} ===");

        let addr: SocketAddrV6 = addr_str.parse().unwrap();
        let encoded = encoder.encode(&addr).unwrap();
        let encoded_str = encoded.to_string();

        println!("Encoded string: '{encoded_str}'");
        println!("String length: {} chars", encoded_str.len());
        println!("String bytes: {:?}", encoded_str.as_bytes());

        println!(
            "split_whitespace(): {:?}",
            encoded_str.split_whitespace().collect::<Vec<_>>()
        );
        println!(
            "split(' '): {:?}",
            encoded_str.split(' ').collect::<Vec<_>>()
        );

        println!(
            "split_whitespace().count(): {}",
            encoded_str.split_whitespace().count()
        );
        println!("split(' ').count(): {}", encoded_str.split(' ').count());

        // Check if there are trailing spaces
        let trimmed = encoded_str.trim_end();
        println!("After trim_end: '{trimmed}'");
        println!("Has trailing spaces: {}", encoded_str != trimmed);
    }
}
