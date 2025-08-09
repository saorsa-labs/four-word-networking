//! Adaptive four-word encoder that handles both IPv4 and IPv6 addresses.
//!
//! This is the main public API for four-word networking.

use crate::dictionary4k::DICTIONARY;
use crate::error::{FourWordError, Result};
use crate::four_word_encoder::FourWordEncoder;
use crate::four_word_ipv6_encoder::{FourWordIpv6Encoder, Ipv6FourWordGroupEncoding};
use crate::validation::{AutocompleteHelper, ValidationResult};
use std::net::{IpAddr, SocketAddr};

/// The main four-word networking encoder interface
pub struct FourWordAdaptiveEncoder {
    ipv4_encoder: FourWordEncoder,
    ipv6_encoder: FourWordIpv6Encoder,
}

impl FourWordAdaptiveEncoder {
    /// Creates a new four-word adaptive encoder
    pub fn new() -> Result<Self> {
        Ok(FourWordAdaptiveEncoder {
            ipv4_encoder: FourWordEncoder::new(),
            ipv6_encoder: FourWordIpv6Encoder::new(),
        })
    }

    /// Encodes any IP address into words
    /// - IPv4: Always exactly 4 words
    /// - IPv6: 6, 9, or 12 words based on compression
    pub fn encode(&self, input: &str) -> Result<String> {
        let addr = self.parse_address(input)?;

        match addr {
            SocketAddr::V4(_) => {
                let encoded = self.ipv4_encoder.encode(addr)?;
                Ok(encoded.to_string())
            }
            SocketAddr::V6(v6) => {
                let encoded = self.ipv6_encoder.encode(&v6)?;
                Ok(encoded.to_string())
            }
        }
    }

    /// Decodes words back to an IP address
    /// Port 65535 is treated as "no port specified" and omitted from output
    pub fn decode(&self, words: &str) -> Result<String> {
        // Determine separator and count words appropriately
        let word_count = if words.contains(' ') {
            // For space-separated words, filter out empty strings from trailing spaces
            words.split(' ').filter(|s| !s.is_empty()).count()
        } else if words.contains('.') {
            // Dot-separated (IPv4 or backwards compatibility)
            words.split('.').filter(|s| !s.is_empty()).count()
        } else if words.contains('-') {
            // Dash-separated (IPv6)
            words.split('-').filter(|s| !s.is_empty()).count()
        } else {
            // Single word or other format
            1
        };

        match word_count {
            4 => {
                // IPv4
                let addr = self.ipv4_encoder.decode(words)?;
                // If port is 65535, omit it (special marker for "no port specified")
                if addr.port() == 65535 {
                    Ok(addr.ip().to_string())
                } else {
                    Ok(addr.to_string())
                }
            }
            6 | 9 | 12 => {
                // IPv6 (6, 9, or 12 words including padding)
                let groups = self.parse_ipv6_groups(words)?;
                let decoded = self.ipv6_encoder.decode(&groups)?;
                let socket_addr = SocketAddr::V6(decoded);
                // If port is 65535, omit it (special marker for "no port specified")
                if socket_addr.port() == 65535 {
                    Ok(socket_addr.ip().to_string())
                } else {
                    Ok(socket_addr.to_string())
                }
            }
            _ => Err(FourWordError::InvalidWordCount {
                expected: 4, // or 6/8/12 for IPv6
                actual: word_count,
            }),
        }
    }

    /// Returns information about the encoding
    pub fn analyze(&self, input: &str) -> Result<String> {
        let addr = self.parse_address(input)?;

        match addr {
            SocketAddr::V4(v4) => Ok(format!(
                "IPv4 Address: {v4}\nEncoding: 4 words\nMethod: Perfect reconstruction"
            )),
            SocketAddr::V6(v6) => {
                let encoded = self.ipv6_encoder.encode(&v6)?;
                Ok(format!(
                    "IPv6 Address: {v6}\nCategory: {:?}\nEncoding: {} words\nMethod: Category-based compression",
                    encoded.category(),
                    encoded.word_count()
                ))
            }
        }
    }

    /// Parses an IP address string into a SocketAddr
    /// Uses port 65535 as a special marker for "no port specified"
    fn parse_address(&self, input: &str) -> Result<SocketAddr> {
        // Try parsing as socket address first
        if let Ok(addr) = input.parse::<SocketAddr>() {
            return Ok(addr);
        }

        // Try parsing as IP address (use port 65535 as marker for "no port specified")
        if let Ok(ip) = input.parse::<IpAddr>() {
            return Ok(match ip {
                IpAddr::V4(v4) => SocketAddr::new(IpAddr::V4(v4), 65535),
                IpAddr::V6(v6) => SocketAddr::new(IpAddr::V6(v6), 65535),
            });
        }

        Err(FourWordError::InvalidInput(format!(
            "Invalid IP address format: {input}"
        )))
    }

    /// Parses IPv6 word groups from a string
    fn parse_ipv6_groups(&self, words: &str) -> Result<Ipv6FourWordGroupEncoding> {
        use crate::four_word_ipv6_encoder::FourWordGroup;
        use crate::ipv6_compression::Ipv6Category;

        // Parse words and filter out empty strings
        let all_words: Vec<String> = if words.contains(' ') {
            words
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else if words.contains('.') {
            words
                .split('.')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else {
            words
                .split('-')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        // IPv6 can have 6, 9, or 12 words
        if all_words.len() != 6 && all_words.len() != 9 && all_words.len() != 12 {
            return Err(FourWordError::InvalidWordCount {
                expected: 6, // or 9/12
                actual: all_words.len(),
            });
        }

        let mut groups = Vec::new();

        // Create groups of 4 words, handling 6 and 9 word cases properly
        match all_words.len() {
            6 => {
                // For 6 words, create 1 group of 4 and keep the remaining 2 words separately
                // This is handled by creating only 1.5 groups but since we can't have half groups,
                // we'll encode the extra 2 words by creating a partial second group
                groups.push(FourWordGroup::new(
                    all_words[0].clone(),
                    all_words[1].clone(),
                    all_words[2].clone(),
                    all_words[3].clone(),
                ));
                // For the remaining 2 words, we need to store them in the compressed data
                // rather than creating empty padding. The IPv6 encoder will handle this.
                // We'll use a special marker approach in the FourWordGroup structure
                groups.push(FourWordGroup::new(
                    all_words[4].clone(),
                    all_words[5].clone(),
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                ));
            }
            9 => {
                // For 9 words, create 2 groups of 4 and keep the remaining 1 word separately
                groups.push(FourWordGroup::new(
                    all_words[0].clone(),
                    all_words[1].clone(),
                    all_words[2].clone(),
                    all_words[3].clone(),
                ));
                groups.push(FourWordGroup::new(
                    all_words[4].clone(),
                    all_words[5].clone(),
                    all_words[6].clone(),
                    all_words[7].clone(),
                ));
                // For the remaining 1 word, create a partial third group
                groups.push(FourWordGroup::new(
                    all_words[8].clone(),
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                    "__MARKER_END__".to_string(), // Special marker to indicate end of data
                ));
            }
            12 => {
                // For 12 words, create groups of 4
                for chunk in all_words.chunks(4) {
                    groups.push(FourWordGroup::new(
                        chunk.first().cloned().unwrap_or_default(),
                        chunk.get(1).cloned().unwrap_or_default(),
                        chunk.get(2).cloned().unwrap_or_default(),
                        chunk.get(3).cloned().unwrap_or_default(),
                    ));
                }
            }
            _ => {
                return Err(FourWordError::InvalidWordCount {
                    expected: 6,
                    actual: all_words.len(),
                })
            }
        };

        // For decoding, we don't know the category yet, so use a placeholder
        // The actual category will be extracted during decoding from the encoded data
        Ok(Ipv6FourWordGroupEncoding::new(
            groups,
            Ipv6Category::GlobalUnicast, // placeholder - will be replaced during decoding
        ))
    }

    // ========== Autocomplete & Hints API ==========

    /// Get word hints for a given prefix
    ///
    /// Returns all dictionary words that start with the given prefix.
    /// With 5 characters, typically returns exactly one word due to unique prefix guarantee.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // Get hints for 3-character prefix
    /// let hints = encoder.get_word_hints("bea");
    /// assert!(!hints.is_empty());
    /// assert!(hints.iter().all(|w| w.starts_with("bea")));
    ///
    /// // With 5 characters, should get unique match
    /// let hints = encoder.get_word_hints("beach");
    /// assert!(hints.len() <= 1);
    /// ```
    pub fn get_word_hints(&self, prefix: &str) -> Vec<String> {
        AutocompleteHelper::get_word_hints(prefix)
    }

    /// Validate partial input and provide suggestions
    ///
    /// Analyzes partial word input to determine validity and provide completions.
    /// Returns information about word count, validity, and possible completions.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // Validate partial input
    /// let result = encoder.validate_partial_input("about abo").unwrap();
    /// assert!(result.is_valid_prefix);
    /// assert_eq!(result.word_count_so_far, 1);
    /// assert!(!result.possible_completions.is_empty());
    /// ```
    pub fn validate_partial_input(&self, partial: &str) -> Result<ValidationResult> {
        AutocompleteHelper::validate_partial_input(partial)
    }

    /// Suggest completions for partial word sequences
    ///
    /// Returns up to 10 complete suggestions based on partial input.
    /// Useful for implementing dropdown autocomplete in UIs.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // Get suggestions for partial input with complete word + partial
    /// let suggestions = encoder.suggest_completions("about ab").unwrap();
    /// assert!(!suggestions.is_empty());
    ///
    /// // Get suggestions for just a partial word
    /// let partial_suggestions = encoder.suggest_completions("abo").unwrap();
    /// assert!(!partial_suggestions.is_empty());
    /// ```
    pub fn suggest_completions(&self, partial_words: &str) -> Result<Vec<String>> {
        AutocompleteHelper::suggest_completions(partial_words)
    }

    /// Auto-complete a word if it has a unique 5-character prefix
    ///
    /// Returns the complete word if the prefix uniquely identifies it.
    /// This enables instant completion when users type 5 characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // With 5+ unique characters
    /// let word = encoder.auto_complete_at_five("beach");
    /// assert_eq!(word, Some("beach".to_string()));
    ///
    /// // With less than 5 characters
    /// let word = encoder.auto_complete_at_five("bea");
    /// assert_eq!(word, None);
    /// ```
    pub fn auto_complete_at_five(&self, prefix: &str) -> Option<String> {
        AutocompleteHelper::auto_complete_at_five(prefix)
    }

    /// Suggest corrections for potentially misspelled words
    ///
    /// Returns suggested corrections for words not found in the dictionary.
    /// Uses prefix matching to find similar words.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // Get corrections for misspelled word
    /// let corrections = encoder.suggest_corrections("aboot");
    /// assert!(!corrections.is_empty());
    ///
    /// // Valid word returns itself
    /// let corrections = encoder.suggest_corrections("about");
    /// assert_eq!(corrections.first(), Some(&"about".to_string()));
    /// ```
    pub fn suggest_corrections(&self, word: &str) -> Vec<String> {
        AutocompleteHelper::suggest_corrections(word)
    }

    /// Check if a string is a valid word prefix
    ///
    /// Returns true if the prefix matches at least one dictionary word.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// assert!(encoder.is_valid_prefix("abo"));
    /// assert!(encoder.is_valid_prefix("about"));
    /// assert!(!encoder.is_valid_prefix("xyz"));
    /// ```
    pub fn is_valid_prefix(&self, prefix: &str) -> bool {
        DICTIONARY.is_valid_prefix(prefix)
    }

    /// Get statistics about possible completions
    ///
    /// Returns information about how many words match a given prefix.
    /// Useful for UI feedback about autocomplete possibilities.
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// let (count, unique_at_five) = encoder.get_completion_stats("abo");
    /// assert!(count > 0);
    /// // unique_at_five indicates if typing 2 more chars will give unique match
    /// ```
    pub fn get_completion_stats(&self, prefix: &str) -> (usize, bool) {
        let hints = self.get_word_hints(prefix);
        let count = hints.len();

        // Check if extending to 5 characters would give unique results
        let unique_at_five = if prefix.len() < 5 {
            // Check if all hints have unique 5-character prefixes
            let mut five_char_prefixes = std::collections::HashSet::new();
            for hint in &hints {
                if hint.len() >= 5 {
                    five_char_prefixes.insert(&hint[..5]);
                }
            }
            five_char_prefixes.len() == hints.len()
        } else {
            count <= 1
        };

        (count, unique_at_five)
    }

    /// Generate random words from the dictionary
    ///
    /// Returns a vector of randomly selected words from the 4096-word dictionary.
    /// These are just random dictionary words, NOT necessarily valid IP encodings.
    /// Useful for generating passphrases, test data, or other applications that
    /// need random English words.
    ///
    /// Each word is independently selected, so duplicates are possible.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of random words to generate
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // Generate 4 random words (not necessarily a valid IP encoding)
    /// let words = encoder.get_random_words(4);
    /// assert_eq!(words.len(), 4);
    ///
    /// // Generate any number of random words
    /// let passphrase = encoder.get_random_words(6);
    /// println!("Random passphrase: {}", passphrase.join("-"));
    ///
    /// // All generated words are valid dictionary words
    /// for word in &words {
    ///     assert!(encoder.is_valid_word(word));
    /// }
    /// ```
    pub fn get_random_words(&self, count: usize) -> Vec<String> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        // Get all words from dictionary
        let all_words = DICTIONARY.get_all_words();

        // Sample random words
        let mut result = Vec::new();
        for _ in 0..count {
            if let Some(word) = all_words.choose(&mut rng) {
                result.push(word.clone());
            }
        }

        result
    }

    /// Check if a word is valid in the dictionary
    ///
    /// Validates whether a given word exists in the 4096-word dictionary.
    /// The check is case-insensitive - uppercase and mixed-case words are
    /// automatically converted to lowercase before validation.
    ///
    /// # Arguments
    ///
    /// * `word` - The word to validate
    ///
    /// # Returns
    ///
    /// * `true` if the word exists in the dictionary
    /// * `false` if the word is not found or contains invalid characters
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::FourWordAdaptiveEncoder;
    ///
    /// let encoder = FourWordAdaptiveEncoder::new().unwrap();
    ///
    /// // Valid words return true
    /// assert!(encoder.is_valid_word("about"));
    /// assert!(encoder.is_valid_word("a"));  // Single-letter words are valid
    ///
    /// // Case-insensitive validation
    /// assert!(encoder.is_valid_word("About"));
    /// assert!(encoder.is_valid_word("ABOUT"));
    ///
    /// // Invalid words return false
    /// assert!(!encoder.is_valid_word("xyz123"));
    /// assert!(!encoder.is_valid_word("not-a-word"));
    /// assert!(!encoder.is_valid_word(""));
    /// ```
    pub fn is_valid_word(&self, word: &str) -> bool {
        DICTIONARY.get_index(word).is_some()
    }
}

impl Default for FourWordAdaptiveEncoder {
    fn default() -> Self {
        Self::new().expect("Failed to create encoder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_encoding() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        let test_cases = vec![
            "192.168.1.1:443",
            "10.0.0.1:80",
            "127.0.0.1:8080",
            "172.16.0.1:22",
        ];

        for addr in test_cases {
            let encoded = encoder.encode(addr).unwrap();
            let word_count = encoded.split_whitespace().count();
            assert_eq!(word_count, 4);

            let decoded = encoder.decode(&encoded).unwrap();
            assert_eq!(addr, decoded);
        }
    }

    #[test]
    fn test_ipv6_encoding() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        let test_cases = vec!["[::1]:443", "[fe80::1]:22", "[2001:db8::1]:8080"];

        for addr in test_cases {
            let encoded = encoder.encode(addr).unwrap();
            let word_count = encoded.split_whitespace().count();
            assert!(word_count == 6 || word_count == 9 || word_count == 12);
            assert!(word_count == 6 || word_count == 9 || word_count == 12); // 6, 9, or 12 words

            let decoded = encoder.decode(&encoded).unwrap();
            // Compare just the IP and port (not the full format)
            let original: SocketAddr = addr.parse().unwrap();
            let decoded_addr: SocketAddr = decoded.parse().unwrap();
            assert_eq!(original.ip(), decoded_addr.ip());
            assert_eq!(original.port(), decoded_addr.port());
        }
    }

    #[test]
    fn test_analyze() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        let analysis = encoder.analyze("192.168.1.1:443").unwrap();
        assert!(analysis.contains("IPv4"));
        assert!(analysis.contains("4 words"));

        let analysis = encoder.analyze("[::1]:443").unwrap();
        assert!(analysis.contains("IPv6"));
        assert!(analysis.contains("words"));
    }

    #[test]
    fn test_get_random_words_basic() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Test requesting different counts
        let words_4 = encoder.get_random_words(4);
        assert_eq!(words_4.len(), 4);

        let words_1 = encoder.get_random_words(1);
        assert_eq!(words_1.len(), 1);

        let words_10 = encoder.get_random_words(10);
        assert_eq!(words_10.len(), 10);

        // Test edge cases
        let words_0 = encoder.get_random_words(0);
        assert_eq!(words_0.len(), 0);

        let words_100 = encoder.get_random_words(100);
        assert_eq!(words_100.len(), 100);
    }

    #[test]
    fn test_random_words_are_valid() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Generate random words and verify they're all valid
        let words = encoder.get_random_words(50);
        for word in &words {
            assert!(
                encoder.is_valid_word(word),
                "Random word '{word}' should be valid"
            );
        }
    }

    #[test]
    fn test_random_words_randomness() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Generate multiple sets and check they're different
        let set1 = encoder.get_random_words(10);
        let set2 = encoder.get_random_words(10);
        let set3 = encoder.get_random_words(10);

        // At least one set should be different from another
        // (statistically, they should all be different)
        assert!(
            set1 != set2 || set2 != set3 || set1 != set3,
            "Random word sets should vary"
        );
    }

    #[test]
    fn test_is_valid_word_basic() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Test with known valid words from the dictionary
        assert!(encoder.is_valid_word("a"));
        assert!(encoder.is_valid_word("about"));

        // Test case insensitivity
        assert!(encoder.is_valid_word("About"));
        assert!(encoder.is_valid_word("ABOUT"));

        // Test invalid words
        assert!(!encoder.is_valid_word("xyz123"));
        assert!(!encoder.is_valid_word("notaword"));
        assert!(!encoder.is_valid_word(""));
        assert!(!encoder.is_valid_word("123"));
        assert!(!encoder.is_valid_word("test-word"));
    }

    #[test]
    fn test_is_valid_word_all_dictionary_words() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Get all dictionary words and verify each is valid
        use crate::dictionary4k::DICTIONARY;
        for i in 0..4096 {
            if let Some(word) = DICTIONARY.get_word(i) {
                assert!(
                    encoder.is_valid_word(word),
                    "Dictionary word '{word}' at index {i} should be valid"
                );
            }
        }
    }

    #[test]
    fn test_is_valid_word_with_special_chars() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Words with special characters should be invalid
        assert!(!encoder.is_valid_word("hello-world"));
        assert!(!encoder.is_valid_word("hello_world"));
        assert!(!encoder.is_valid_word("hello.world"));
        assert!(!encoder.is_valid_word("hello world"));
        assert!(!encoder.is_valid_word("hello@world"));
        assert!(!encoder.is_valid_word("hello!"));
    }

    #[test]
    fn test_random_words_are_just_dictionary_words() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // get_random_words() should return random dictionary words
        // These are NOT necessarily valid IP encodings - just random words for other uses
        // like generating passphrases, test data, etc.

        let words_4 = encoder.get_random_words(4);
        assert_eq!(words_4.len(), 4);

        // Verify they're all from the dictionary
        for word in &words_4 {
            assert!(encoder.is_valid_word(word));
        }

        // Can generate any number of random words
        let words_7 = encoder.get_random_words(7);
        assert_eq!(words_7.len(), 7);

        let words_20 = encoder.get_random_words(20);
        assert_eq!(words_20.len(), 20);

        // These random words are NOT meant to be decoded as IP addresses
        // They're just random dictionary words for general use
    }

    #[test]
    fn test_is_valid_word_edge_cases() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();

        // Test single-letter words (should have some valid ones like "a", "i")
        assert!(encoder.is_valid_word("a"));

        // Test very long invalid strings
        let long_word = "a".repeat(100);
        assert!(!encoder.is_valid_word(&long_word));

        // Test whitespace
        assert!(!encoder.is_valid_word(" "));
        assert!(!encoder.is_valid_word("\t"));
        assert!(!encoder.is_valid_word("\n"));

        // Test words with leading/trailing spaces (should work due to lowercase conversion)
        // The dictionary lookup uses to_lowercase which doesn't trim
        assert!(!encoder.is_valid_word(" about "));
        assert!(!encoder.is_valid_word("about "));
        assert!(!encoder.is_valid_word(" about"));
    }

    #[test]
    fn test_random_words_distribution() {
        let encoder = FourWordAdaptiveEncoder::new().unwrap();
        use std::collections::HashSet;

        // Generate a large sample and check distribution
        let sample_size = 1000;
        let words = encoder.get_random_words(sample_size);

        // Convert to set to count unique words
        let unique_words: HashSet<_> = words.iter().collect();

        // We should see a reasonable variety of words
        // With 4096 words and 1000 samples, we expect to see several hundred unique words
        assert!(
            unique_words.len() > 100,
            "Random selection should produce variety, got {} unique words from {} samples",
            unique_words.len(),
            sample_size
        );

        // But not all unique (that would suggest non-random behavior)
        assert!(
            unique_words.len() < sample_size,
            "Some repetition expected in random selection"
        );
    }
}
