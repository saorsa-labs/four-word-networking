use four_word_networking::FourWordAdaptiveEncoder;

fn main() {
    println!("=== Testing IPv6 addresses from integration tests ===\n");

    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    let test_addresses = vec![
        "::1",
        "::",
        "2001:db8::1",
        "fe80::1",
        "ff02::1",
        "2001:4860:4860::8888",
        "2606:4700:4700::1111",
    ];

    for addr in test_addresses {
        println!("\nTesting: {addr}");

        match encoder.encode(addr) {
            Ok(encoded) => {
                println!("  Encoded: '{encoded}'");
                let word_count = encoded.split_whitespace().count();
                println!("  Word count: {word_count}");

                match encoder.decode(&encoded) {
                    Ok(decoded) => {
                        println!("  Decoded: '{decoded}'");
                        if decoded == addr {
                            println!("  ✓ Exact match!");
                        } else if decoded == format!("[{addr}]:0") {
                            println!("  ✓ Match with added port (expected behavior)");
                        } else {
                            println!("  ✗ MISMATCH!");
                        }
                    }
                    Err(e) => {
                        println!("  Decode error: {e:?}");
                    }
                }
            }
            Err(e) => {
                println!("  Encode error: {e:?}");
            }
        }
    }
}
