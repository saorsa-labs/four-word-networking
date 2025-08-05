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
