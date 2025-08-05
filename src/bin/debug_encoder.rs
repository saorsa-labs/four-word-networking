use four_word_networking::FourWordAdaptiveEncoder;
use std::net::Ipv4Addr;

fn main() {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let addr = "0.148.217.0:0";
    println!("Testing address: {addr}");

    // Let's manually debug the encoding process
    let ipv4: Ipv4Addr = "0.148.217.0".parse().unwrap();
    let port = 0u16;
    println!("IPv4: {ipv4:?}, Port: {port}");

    let octets = ipv4.octets();
    println!("Octets: {octets:?}");

    // Pack the 48 bits: IPv4 (32 bits) + port (16 bits)
    let mut bytes = [0u8; 6];
    bytes[0..4].copy_from_slice(&octets);
    bytes[4..6].copy_from_slice(&port.to_be_bytes());
    println!("Packed bytes: {bytes:?}");

    // Convert to 48-bit integer
    let mut n = 0u64;
    for byte in bytes {
        n = (n << 8) | (byte as u64);
    }
    println!("48-bit integer: {n}");

    // Debug the word extraction
    let mut words = Vec::with_capacity(4);
    let mut remaining = n;
    println!("Starting extraction:");

    for i in 0..4 {
        let index = (remaining % 4096) as u16;
        println!("  Iteration {i}: remaining={remaining}, index={index}");
        words.push(index);
        remaining /= 4096;
    }
    println!("Word indices: {words:?}");
    println!("Remaining after 4 iterations: {remaining}");

    // Let's also check if the word indices match what we expect
    use four_word_networking::dictionary4k::DICTIONARY;
    for &index in words.iter() {
        if let Some(word) = DICTIONARY.get_word(index) {
            println!("Index {index} -> '{word}'");
        }
    }

    // Test with adaptive encoder
    match encoder.encode(addr) {
        Ok(words) => {
            println!("Adaptive encoder words: '{words}'");
            let word_vec: Vec<&str> = words.split_whitespace().collect();
            println!("Adaptive word count: {}", word_vec.len());
            for (i, word) in word_vec.iter().enumerate() {
                println!("  {}: '{}'", i + 1, word);
            }
        }
        Err(e) => println!("Adaptive error: {e}"),
    }
}
