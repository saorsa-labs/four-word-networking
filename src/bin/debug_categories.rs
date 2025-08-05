use four_word_networking::{FourWordIpv6Encoder, Ipv6Compressor};
use std::net::SocketAddrV6;

fn main() {
    let compressor = Ipv6Compressor::new();
    let encoder = FourWordIpv6Encoder::new();

    let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

    for addr_str in test_cases {
        println!("\n=== Testing address: {addr_str} ===");

        let addr: SocketAddrV6 = addr_str.parse().unwrap();
        println!("Parsed IPv6: {} port: {}", addr.ip(), addr.port());

        // Check what category the compressor assigns
        match compressor.compress(*addr.ip(), Some(addr.port())) {
            Ok(compressed) => {
                println!("Category: {:?}", compressed.category);
                println!(
                    "Compressed data length: {} bytes",
                    compressed.compressed_data.len()
                );
                println!("Compressed data: {:?}", compressed.compressed_data);

                // Try encoding
                match encoder.encode(&addr) {
                    Ok(encoded) => {
                        println!("Encoded successfully: {} words", encoded.word_count());
                        println!("Encoded category: {:?}", encoded.category());

                        // Try decoding
                        match encoder.decode(&encoded) {
                            Ok(decoded) => {
                                println!(
                                    "Decoded successfully: {}:{}",
                                    decoded.ip(),
                                    decoded.port()
                                );
                            }
                            Err(e) => {
                                println!("Decode error: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        println!("Encode error: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Compression error: {e}");
            }
        }
    }
}
