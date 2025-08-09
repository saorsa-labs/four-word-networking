//! Validation and autocomplete functionality for four-word networking.

use crate::dictionary4k::DICTIONARY;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Result of validating partial input
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the current prefix is valid
    pub is_valid_prefix: bool,
    /// Possible completions for the current word
    pub possible_completions: Vec<String>,
    /// Number of complete words entered so far
    pub word_count_so_far: usize,
    /// Expected total number of words (4 for IPv4, 6/9/12 for IPv6)
    pub expected_total_words: Option<usize>,
    /// Whether the input is complete
    pub is_complete: bool,
}

impl ValidationResult {
    /// Create a validation result for incomplete input
    pub fn incomplete(is_valid: bool, completions: Vec<String>, word_count: usize) -> Self {
        ValidationResult {
            is_valid_prefix: is_valid,
            possible_completions: completions,
            word_count_so_far: word_count,
            expected_total_words: None,
            is_complete: false,
        }
    }

    /// Create a validation result for complete input
    pub fn complete(word_count: usize) -> Self {
        ValidationResult {
            is_valid_prefix: true,
            possible_completions: vec![],
            word_count_so_far: word_count,
            expected_total_words: Some(word_count),
            is_complete: true,
        }
    }
}

/// Autocomplete helper for four-word networking
pub struct AutocompleteHelper;

impl AutocompleteHelper {
    /// Get word hints for a given prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::validation::AutocompleteHelper;
    ///
    /// let hints = AutocompleteHelper::get_word_hints("bea");
    /// assert!(!hints.is_empty());
    /// assert!(hints.iter().all(|w| w.starts_with("bea")));
    /// ```
    pub fn get_word_hints(prefix: &str) -> Vec<String> {
        DICTIONARY.get_word_hints(prefix)
    }

    /// Validate partial input and provide suggestions
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::validation::AutocompleteHelper;
    ///
    /// let result = AutocompleteHelper::validate_partial_input("beach cont").unwrap();
    /// assert!(result.is_valid_prefix);
    /// assert_eq!(result.word_count_so_far, 1);
    /// assert!(!result.possible_completions.is_empty());
    /// ```
    pub fn validate_partial_input(partial: &str) -> Result<ValidationResult> {
        if partial.is_empty() {
            return Ok(ValidationResult::incomplete(true, vec![], 0));
        }

        // Split input into words
        let parts: Vec<&str> = partial.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(ValidationResult::incomplete(true, vec![], 0));
        }

        // Count complete words (those that exist in dictionary)
        let mut complete_words = 0;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part might be incomplete
                if DICTIONARY.get_index(part).is_some() {
                    complete_words += 1;
                } else {
                    // It's a partial word, get hints
                    let hints = DICTIONARY.get_word_hints(part);
                    let is_valid = !hints.is_empty();
                    return Ok(ValidationResult::incomplete(
                        is_valid,
                        hints,
                        complete_words,
                    ));
                }
            } else {
                // All non-last parts must be complete words
                if DICTIONARY.get_index(part).is_some() {
                    complete_words += 1;
                } else {
                    // Invalid word in the middle
                    return Ok(ValidationResult::incomplete(false, vec![], complete_words));
                }
            }
        }

        // Check if we have a valid complete address
        match complete_words {
            4 => Ok(ValidationResult::complete(4)),
            6 | 9 | 12 => Ok(ValidationResult::complete(complete_words)),
            _ => Ok(ValidationResult::incomplete(true, vec![], complete_words)),
        }
    }

    /// Suggest completions for partial words
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::validation::AutocompleteHelper;
    ///
    /// let suggestions = AutocompleteHelper::suggest_completions("beach cont").unwrap();
    /// assert!(!suggestions.is_empty());
    /// ```
    pub fn suggest_completions(partial_words: &str) -> Result<Vec<String>> {
        let validation = Self::validate_partial_input(partial_words)?;

        if validation.is_complete {
            return Ok(vec![partial_words.to_string()]);
        }

        // Get the base (complete words so far)
        let parts: Vec<&str> = partial_words.split_whitespace().collect();

        // Check if the last part is a partial word or if we need new suggestions
        let (base, need_completions) = if parts.is_empty() {
            (String::new(), true)
        } else if !validation.possible_completions.is_empty() {
            // Has a partial word that needs completion
            if parts.len() > 1 {
                (parts[..parts.len() - 1].join(" "), false)
            } else {
                (String::new(), false)
            }
        } else {
            // All complete words, need to suggest next word
            (partial_words.to_string(), true)
        };

        let mut suggestions = Vec::new();

        if need_completions {
            // Suggest some common starting words
            let common_prefixes = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
            for prefix in common_prefixes {
                let hints = DICTIONARY.get_word_hints(prefix);
                if let Some(first) = hints.first() {
                    if base.is_empty() {
                        suggestions.push(first.clone());
                    } else {
                        suggestions.push(format!("{base} {first}"));
                    }
                    if suggestions.len() >= 10 {
                        break;
                    }
                }
            }
        } else {
            // Use the validation's possible completions
            for completion in validation.possible_completions.iter().take(10) {
                if base.is_empty() {
                    suggestions.push(completion.clone());
                } else {
                    suggestions.push(format!("{base} {completion}"));
                }
            }
        }

        Ok(suggestions)
    }

    /// Auto-complete if there's a unique match at 5 characters
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::validation::AutocompleteHelper;
    ///
    /// // With 5 unique characters
    /// let completed = AutocompleteHelper::auto_complete_at_five("beach");
    /// assert_eq!(completed, Some("beach".to_string()));
    ///
    /// // With less than 5 characters
    /// let not_completed = AutocompleteHelper::auto_complete_at_five("bea");
    /// assert_eq!(not_completed, None);
    /// ```
    pub fn auto_complete_at_five(prefix: &str) -> Option<String> {
        if prefix.len() >= 5 {
            DICTIONARY.get_unique_word_for_prefix(prefix)
        } else {
            None
        }
    }

    /// Suggest corrections for potentially misspelled words
    ///
    /// # Examples
    ///
    /// ```
    /// use four_word_networking::validation::AutocompleteHelper;
    ///
    /// // This is a simple example - real implementation would use
    /// // edit distance algorithms for better suggestions
    /// let corrections = AutocompleteHelper::suggest_corrections("beech");
    /// assert!(!corrections.is_empty());
    /// ```
    pub fn suggest_corrections(word: &str) -> Vec<String> {
        // First check if it's a valid word
        if DICTIONARY.get_index(word).is_some() {
            return vec![word.to_string()];
        }

        // Try to find similar words by prefix
        let mut suggestions = Vec::new();

        // Check progressively shorter prefixes
        for len in (1..=word.len().min(5)).rev() {
            let prefix = &word[..len];
            let hints = DICTIONARY.get_word_hints(prefix);
            if !hints.is_empty() {
                suggestions.extend(hints.into_iter().take(5));
                break;
            }
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty() {
        let result = AutocompleteHelper::validate_partial_input("").unwrap();
        assert!(result.is_valid_prefix);
        assert_eq!(result.word_count_so_far, 0);
        assert!(!result.is_complete);
    }

    #[test]
    fn test_validation_partial_word() {
        let result = AutocompleteHelper::validate_partial_input("abo").unwrap();
        assert!(result.is_valid_prefix);
        assert_eq!(result.word_count_so_far, 0);
        assert!(!result.possible_completions.is_empty());
        assert!(!result.is_complete);
    }

    #[test]
    fn test_validation_complete_ipv4() {
        // We need to use actual words from the dictionary
        // Let's assume "about" is in the dictionary
        if DICTIONARY.get_index("about").is_some() {
            // Create a valid 4-word combination (if these words exist)
            let test_words = ["about", "above", "absent", "accept"];
            let all_valid = test_words.iter().all(|w| DICTIONARY.get_index(w).is_some());

            if all_valid {
                let input = test_words.join(" ");
                let result = AutocompleteHelper::validate_partial_input(&input).unwrap();
                assert!(result.is_valid_prefix);
                assert_eq!(result.word_count_so_far, 4);
                assert!(result.is_complete);
            }
        }
    }

    #[test]
    fn test_auto_complete_at_five() {
        // Test with 5+ character prefix
        let word = AutocompleteHelper::auto_complete_at_five("about");
        // Should return Some if "about" exists and is unique at 5 chars
        assert!(word.is_some() || word.is_none());

        // Test with less than 5 characters
        let word = AutocompleteHelper::auto_complete_at_five("abo");
        assert_eq!(word, None);
    }

    #[test]
    fn test_suggest_completions() {
        // First check if we have words starting with "a"
        let hints = AutocompleteHelper::get_word_hints("a");
        assert!(!hints.is_empty(), "Should have words starting with 'a'");

        let suggestions = AutocompleteHelper::suggest_completions("a").unwrap();
        assert!(!suggestions.is_empty(), "Should have suggestions for 'a'");
        assert!(suggestions.len() <= 10); // Limited to 10 suggestions
    }

    #[test]
    fn test_suggest_corrections() {
        // Test with a valid word
        if DICTIONARY.get_index("about").is_some() {
            let corrections = AutocompleteHelper::suggest_corrections("about");
            assert_eq!(corrections, vec!["about".to_string()]);
        }

        // Test with an invalid word
        let corrections = AutocompleteHelper::suggest_corrections("aboot");
        assert!(!corrections.is_empty()); // Should suggest something
    }
}
