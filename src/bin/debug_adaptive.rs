use four_word_networking::FourWordAdaptiveEncoder;

fn main() {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

    for addr in test_cases {
        println!("\n=== Testing adaptive encoder with: {addr} ===");

        match encoder.encode(addr) {
            Ok(encoded) => {
                println!("Encoded: '{encoded}'");
                let word_count = encoded.split_whitespace().count();
                println!("Word count: {word_count}");

                match encoder.decode(&encoded) {
                    Ok(decoded) => {
                        println!("Decoded: '{decoded}'");
                        println!("Success!");
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
}
