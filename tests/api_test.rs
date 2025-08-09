//! Integration tests for the library API
//!
//! Tests the public API methods for random word generation and word validation.

use four_word_networking::FourWordAdaptiveEncoder;

#[test]
fn test_random_words_api() {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    // Generate random words
    let words = encoder.get_random_words(4);
    assert_eq!(words.len(), 4);

    // All words should be valid
    for word in &words {
        assert!(
            encoder.is_valid_word(word),
            "Generated word '{word}' should be valid"
        );
    }
}

#[test]
fn test_word_validation_api() {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    // Test valid words
    assert!(encoder.is_valid_word("a"));
    assert!(encoder.is_valid_word("about"));
    assert!(encoder.is_valid_word("ABOUT")); // Case insensitive

    // Test invalid words
    assert!(!encoder.is_valid_word("notaword"));
    assert!(!encoder.is_valid_word("123"));
    assert!(!encoder.is_valid_word(""));
}

#[test]
fn test_random_words_for_passphrases() {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    // get_random_words() is for generating random dictionary words
    // NOT for generating valid IP encodings

    // Example: Generate a memorable passphrase
    let passphrase_words = encoder.get_random_words(6);
    assert_eq!(passphrase_words.len(), 6);
    let passphrase = passphrase_words.join("-");

    // Verify all words are from the dictionary
    for word in &passphrase_words {
        assert!(encoder.is_valid_word(word));
    }

    // The passphrase is just random words, not meant to decode to an IP
    println!("Example passphrase: {passphrase}");
}

#[test]
fn test_validate_generated_words() {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    // Generate a large batch and validate all
    let batch_size = 100;
    let words = encoder.get_random_words(batch_size);

    let valid_count = words.iter().filter(|w| encoder.is_valid_word(w)).count();
    assert_eq!(
        valid_count, batch_size,
        "All generated words should be valid"
    );
}

#[test]
fn test_api_example_usage() {
    // Example of how a user might use the API
    let encoder = FourWordAdaptiveEncoder::new().unwrap();

    // User wants to generate a random passphrase
    let passphrase_words = encoder.get_random_words(6);
    println!("Random passphrase: {}", passphrase_words.join("-"));

    // User wants to validate words from user input
    let user_input = vec!["ocean", "blue", "mountain", "river"];
    let mut all_valid = true;
    for word in &user_input {
        if !encoder.is_valid_word(word) {
            println!("Invalid word: {word}");
            all_valid = false;
        }
    }

    if all_valid {
        // Try to decode if we have the right number of words
        if user_input.len() == 4 {
            let word_string = user_input.join(" ");
            match encoder.decode(&word_string) {
                Ok(address) => println!("Decoded to: {address}"),
                Err(e) => println!("Not a valid encoding: {e}"),
            }
        }
    }
}
