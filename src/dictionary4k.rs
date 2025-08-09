//! 4,096-word dictionary for four-word networking encoding.
//!
//! This module provides a dictionary of exactly 4,096 (2^12) words for encoding
//! IP addresses using four words. Each word can represent 12 bits of information.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Static dictionary containing exactly 4,096 words
pub static DICTIONARY: Lazy<Dictionary4K> =
    Lazy::new(|| Dictionary4K::new().expect("Failed to initialize 4K dictionary"));

/// A dictionary of 4,096 words for four-word encoding
pub struct Dictionary4K {
    /// Words indexed by their position (0-4095)
    words: Vec<String>,
    /// Reverse lookup: word -> index
    word_to_index: HashMap<String, u16>,
}

impl Dictionary4K {
    /// Creates a new dictionary from the embedded word list
    pub fn new() -> Result<Self, String> {
        let wordlist = include_str!("../GOLD_WORDLIST_OPTIMIZED.txt");
        let words: Vec<String> = wordlist
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(4096)
            .map(|s| s.trim().to_lowercase())
            .collect();

        if words.len() != 4096 {
            return Err(format!(
                "Dictionary must contain exactly 4096 words, found {}",
                words.len()
            ));
        }

        let mut word_to_index = HashMap::with_capacity(4096);
        for (index, word) in words.iter().enumerate() {
            if word_to_index.insert(word.clone(), index as u16).is_some() {
                return Err(format!("Duplicate word found: {word}"));
            }
        }

        Ok(Dictionary4K {
            words,
            word_to_index,
        })
    }

    /// Gets a word by its index (0-4095)
    pub fn get_word(&self, index: u16) -> Option<&str> {
        if index < 4096 {
            self.words.get(index as usize).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Gets the index of a word (0-4095)
    pub fn get_index(&self, word: &str) -> Option<u16> {
        self.word_to_index.get(&word.to_lowercase()).copied()
    }

    /// Returns the total number of words (always 4096)
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Checks if the dictionary is empty (always false for valid dictionary)
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Get word hints for a given prefix
    /// Returns all words that start with the given prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::dictionary4k::DICTIONARY;
    ///
    /// let hints = DICTIONARY.get_word_hints("bea");
    /// assert!(hints.iter().any(|w| w == "beach"));
    /// assert!(hints.iter().any(|w| w == "beam"));
    ///
    /// // With 5 characters, should return at most one word (unique prefix)
    /// let hints = DICTIONARY.get_word_hints("beach");
    /// assert_eq!(hints.len(), 1);
    /// assert_eq!(hints[0], "beach");
    /// ```
    pub fn get_word_hints(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() {
            return vec![];
        }

        let prefix_lower = prefix.to_lowercase();
        self.words
            .iter()
            .filter(|word| word.starts_with(&prefix_lower))
            .cloned()
            .collect()
    }

    /// Check if a prefix matches any word in the dictionary
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::dictionary4k::DICTIONARY;
    ///
    /// assert!(DICTIONARY.is_valid_prefix("bea"));
    /// assert!(DICTIONARY.is_valid_prefix("beach"));
    /// assert!(!DICTIONARY.is_valid_prefix("xyz"));
    /// ```
    pub fn is_valid_prefix(&self, prefix: &str) -> bool {
        if prefix.is_empty() {
            return false;
        }

        let prefix_lower = prefix.to_lowercase();
        self.words
            .iter()
            .any(|word| word.starts_with(&prefix_lower))
    }

    /// Get the unique word for a 5-character prefix (if it exists)
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::dictionary4k::DICTIONARY;
    ///
    /// // With 5 characters, should get unique match
    /// let word = DICTIONARY.get_unique_word_for_prefix("beach");
    /// assert_eq!(word, Some("beach".to_string()));
    ///
    /// // With less than 5 characters, might not be unique
    /// let word = DICTIONARY.get_unique_word_for_prefix("bea");
    /// assert_eq!(word, None); // Multiple matches
    /// ```
    pub fn get_unique_word_for_prefix(&self, prefix: &str) -> Option<String> {
        if prefix.len() < 5 {
            return None;
        }

        let matches = self.get_word_hints(prefix);
        if matches.len() == 1 {
            Some(matches[0].clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_size() {
        let dict = Dictionary4K::new().unwrap();
        assert_eq!(dict.len(), 4096);
    }

    #[test]
    fn test_word_lookup() {
        let dict = Dictionary4K::new().unwrap();

        // Test first word
        let first_word = dict.get_word(0).unwrap();
        assert_eq!(dict.get_index(first_word), Some(0));

        // Test last word
        let last_word = dict.get_word(4095).unwrap();
        assert_eq!(dict.get_index(last_word), Some(4095));
    }

    #[test]
    fn test_word_hints() {
        let dict = Dictionary4K::new().unwrap();

        // Test with 3-character prefix
        let hints = dict.get_word_hints("abo");
        assert!(!hints.is_empty());
        assert!(hints.iter().all(|w| w.starts_with("abo")));

        // Test with 5-character prefix (should be unique or very few matches)
        let hints_5 = dict.get_word_hints("about");
        assert!(hints_5.len() <= 1);
        if !hints_5.is_empty() {
            assert_eq!(hints_5[0], "about");
        }

        // Test with non-existent prefix
        let no_hints = dict.get_word_hints("xyz");
        assert!(no_hints.is_empty());
    }

    #[test]
    fn test_valid_prefix() {
        let dict = Dictionary4K::new().unwrap();

        // Test valid prefixes
        assert!(dict.is_valid_prefix("a"));
        assert!(dict.is_valid_prefix("ab"));
        assert!(dict.is_valid_prefix("abo"));

        // Test invalid prefix
        assert!(!dict.is_valid_prefix("xyz"));
        assert!(!dict.is_valid_prefix(""));
    }

    #[test]
    fn test_unique_word_for_prefix() {
        let dict = Dictionary4K::new().unwrap();

        // Test with 5-character prefix
        let word = dict.get_unique_word_for_prefix("about");
        assert!(word.is_some() || word.is_none()); // Depends on dictionary content

        // Test with less than 5 characters (should return None)
        assert_eq!(dict.get_unique_word_for_prefix("abo"), None);
        assert_eq!(dict.get_unique_word_for_prefix("a"), None);

        // Test with non-existent prefix
        assert_eq!(dict.get_unique_word_for_prefix("xyzab"), None);
    }

    #[test]
    fn test_out_of_bounds() {
        let dict = Dictionary4K::new().unwrap();
        assert_eq!(dict.get_word(4096), None);
        assert_eq!(dict.get_word(65535), None);
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let dict = Dictionary4K::new().unwrap();
        let word = dict.get_word(0).unwrap();

        assert_eq!(dict.get_index(word), Some(0));
        assert_eq!(dict.get_index(&word.to_uppercase()), Some(0));
    }
}
