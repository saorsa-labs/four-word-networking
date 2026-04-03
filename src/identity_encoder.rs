//! Identity word encoder for x0x agent and user identities.
//!
//! This module implements the identity word system described in the x0x
//! FUTURE_PATH specification. It encodes 256-bit cryptographic hashes
//! (AgentId, UserId) into human-speakable four-word names using the first
//! 48 bits as a prefix, mapped through the 4,096-word dictionary.
//!
//! # Identity Types
//!
//! - **Agent identity (4 words)**: An autonomous agent with no human backing.
//!   Derived from the first 48 bits of the AgentId (SHA-256 of ML-DSA-65 public key).
//!
//! - **Full identity (8 words)**: A human-backed agent, formatted as
//!   `agent-words @ user-words`. The `@` separator mirrors email conventions
//!   and carries the semantic "this agent *at* this person."
//!
//! # Word Count Semantics
//!
//! The word count carries meaning:
//! - **4 words** = autonomous agent, no human vouching for it
//! - **8 words (4 @ 4)** = human-backed agent, cryptographically bound to a person
//!
//! # Collision Resistance
//!
//! Each 4-word identity provides 48 bits of prefix from a 256-bit hash.
//! Birthday-bound collision threshold is ~2^24 (~16 million) per half.
//! The combined 8-word identity provides ~2^48 (~281 trillion) collision resistance.
//!
//! # Examples
//!
//! ```rust
//! use four_word_networking::identity_encoder::IdentityEncoder;
//!
//! let encoder = IdentityEncoder::new();
//!
//! // Encode an agent ID (32 bytes) to 4 words
//! let agent_id = hex::decode(
//!     "dd6530452610619d468e4e82be82107e86384365c58efa6e3018d7762c7368da"
//! ).unwrap();
//! let words = encoder.encode_agent(&agent_id).unwrap();
//! println!("Agent: {}", words);  // e.g. "highland forest moon river"
//!
//! // Decode 4 words back to a 48-bit prefix
//! let prefix = encoder.decode_to_prefix(&words.to_string()).unwrap();
//! assert_eq!(&agent_id[..6], &prefix[..]);
//!
//! // Encode a full 8-word identity (agent @ user)
//! let user_id = hex::decode(
//!     "3e729de0469a594d1e042a672b29adde388e34aed2ced1e4c244a87f03053770"
//! ).unwrap();
//! let full = encoder.encode_full(&agent_id, &user_id).unwrap();
//! println!("Identity: {}", full);  // e.g. "highland forest moon river @ castle autumn wind silver"
//! ```

use crate::dictionary4k::DICTIONARY;
use crate::error::{FourWordError, Result};

/// Represents an encoded identity — either 4 words (agent) or 8 words (agent @ user).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityWords {
    /// Autonomous agent identity (4 words from AgentId prefix)
    Agent { words: [String; 4] },
    /// Human-backed agent identity (4 agent words @ 4 user words)
    Full {
        agent_words: [String; 4],
        user_words: [String; 4],
    },
}

impl IdentityWords {
    /// Returns just the agent words (first 4)
    pub fn agent_words(&self) -> &[String; 4] {
        match self {
            IdentityWords::Agent { words } => words,
            IdentityWords::Full { agent_words, .. } => agent_words,
        }
    }

    /// Returns the user words if this is a full identity
    pub fn user_words(&self) -> Option<&[String; 4]> {
        match self {
            IdentityWords::Agent { .. } => None,
            IdentityWords::Full { user_words, .. } => Some(user_words),
        }
    }

    /// Returns true if this is a full (human-backed) identity
    pub fn is_full(&self) -> bool {
        matches!(self, IdentityWords::Full { .. })
    }

    /// Returns the number of identity words (4 or 8)
    pub fn word_count(&self) -> usize {
        match self {
            IdentityWords::Agent { .. } => 4,
            IdentityWords::Full { .. } => 8,
        }
    }
}

impl std::fmt::Display for IdentityWords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityWords::Agent { words } => {
                write!(f, "{}", words.join(" "))
            }
            IdentityWords::Full {
                agent_words,
                user_words,
            } => {
                write!(f, "{} @ {}", agent_words.join(" "), user_words.join(" "))
            }
        }
    }
}

/// Encoder for x0x identity words.
///
/// Converts 256-bit cryptographic hashes (AgentId, UserId) into
/// human-speakable four-word names using the 4,096-word dictionary.
pub struct IdentityEncoder;

impl IdentityEncoder {
    /// Creates a new identity encoder.
    pub fn new() -> Self {
        IdentityEncoder
    }

    /// Encodes the first 48 bits of a hash into 4 words.
    ///
    /// Takes a 32-byte hash (SHA-256 of an ML-DSA-65 public key),
    /// extracts the first 48 bits, and maps them to 4 dictionary words
    /// at 12 bits per word.
    fn encode_hash_prefix(&self, hash: &[u8]) -> Result<[String; 4]> {
        if hash.len() < 6 {
            return Err(FourWordError::InvalidInput(format!(
                "Hash must be at least 6 bytes (48 bits), got {} bytes",
                hash.len()
            )));
        }

        // Extract first 48 bits as a u64
        let mut n: u64 = 0;
        for &byte in &hash[..6] {
            n = (n << 8) | (byte as u64);
        }

        // Split into 4 x 12-bit indices (most significant first)
        let mut words = Vec::with_capacity(4);
        for i in (0..4).rev() {
            let index = ((n >> (i * 12)) & 0xFFF) as u16;
            let word = DICTIONARY
                .get_word(index)
                .ok_or(FourWordError::InvalidWordIndex(index))?
                .to_string();
            words.push(word);
        }

        Ok([
            words[0].clone(),
            words[1].clone(),
            words[2].clone(),
            words[3].clone(),
        ])
    }

    /// Decodes 4 words back to a 48-bit (6-byte) prefix.
    ///
    /// This is the reverse of `encode_hash_prefix`. The returned bytes
    /// can be used as a search prefix to locate agents on the gossip network.
    pub fn decode_to_prefix(&self, identity: &str) -> Result<[u8; 6]> {
        let words: Vec<&str> = identity.split_whitespace().collect();
        if words.len() != 4 {
            return Err(FourWordError::InvalidInput(format!(
                "Expected 4 words, got {}",
                words.len()
            )));
        }

        self.decode_words_to_prefix(&words)
    }

    /// Decodes a slice of 4 word strings to a 48-bit prefix.
    fn decode_words_to_prefix(&self, words: &[&str]) -> Result<[u8; 6]> {
        if words.len() != 4 {
            return Err(FourWordError::InvalidInput(format!(
                "Expected 4 words, got {}",
                words.len()
            )));
        }

        // Reconstruct the 48-bit value from 4 x 12-bit indices
        let mut n: u64 = 0;
        for word in words {
            let index = DICTIONARY
                .get_index(word)
                .ok_or_else(|| FourWordError::InvalidWord(word.to_string()))?;
            n = (n << 12) | (index as u64);
        }

        // Convert to 6 bytes (big-endian)
        let mut prefix = [0u8; 6];
        for (i, byte) in prefix.iter_mut().enumerate() {
            *byte = ((n >> (40 - i * 8)) & 0xFF) as u8;
        }

        Ok(prefix)
    }

    /// Encodes an AgentId into 4 identity words.
    ///
    /// The AgentId is the SHA-256 hash of the agent's ML-DSA-65 public key.
    /// This produces an autonomous agent identity (4 words, no human backing).
    pub fn encode_agent(&self, agent_id: &[u8]) -> Result<IdentityWords> {
        let words = self.encode_hash_prefix(agent_id)?;
        Ok(IdentityWords::Agent { words })
    }

    /// Encodes an AgentId and UserId into 8 identity words (4 @ 4).
    ///
    /// This produces a full human-backed identity. The AgentId is the
    /// agent's key hash, the UserId is the human's key hash.
    pub fn encode_full(&self, agent_id: &[u8], user_id: &[u8]) -> Result<IdentityWords> {
        let agent_words = self.encode_hash_prefix(agent_id)?;
        let user_words = self.encode_hash_prefix(user_id)?;
        Ok(IdentityWords::Full {
            agent_words,
            user_words,
        })
    }

    /// Encodes a hex-encoded hash string into identity words.
    ///
    /// Convenience method that accepts a hex string (as displayed by `x0x agent`).
    pub fn encode_hex(&self, hex_str: &str) -> Result<IdentityWords> {
        let bytes = hex::decode(hex_str.trim())
            .map_err(|e| FourWordError::InvalidInput(format!("Invalid hex string: {e}")))?;
        self.encode_agent(&bytes)
    }

    /// Encodes two hex strings into a full 8-word identity.
    pub fn encode_hex_full(&self, agent_hex: &str, user_hex: &str) -> Result<IdentityWords> {
        let agent_bytes = hex::decode(agent_hex.trim())
            .map_err(|e| FourWordError::InvalidInput(format!("Invalid agent hex: {e}")))?;
        let user_bytes = hex::decode(user_hex.trim())
            .map_err(|e| FourWordError::InvalidInput(format!("Invalid user hex: {e}")))?;
        self.encode_full(&agent_bytes, &user_bytes)
    }

    /// Parses an identity string into `IdentityWords`.
    ///
    /// Accepts either:
    /// - 4 space-separated words (agent identity)
    /// - 8 words with `@` separator (full identity): `"word1 word2 word3 word4 @ word5 word6 word7 word8"`
    pub fn parse(&self, input: &str) -> Result<IdentityWords> {
        if input.contains('@') {
            // Full identity: agent @ user
            let parts: Vec<&str> = input.split('@').collect();
            if parts.len() != 2 {
                return Err(FourWordError::InvalidInput(
                    "Full identity must have exactly one '@' separator".to_string(),
                ));
            }

            let agent_words: Vec<&str> = parts[0].split_whitespace().collect();
            let user_words: Vec<&str> = parts[1].split_whitespace().collect();

            if agent_words.len() != 4 {
                return Err(FourWordError::InvalidInput(format!(
                    "Agent part must have 4 words, got {}",
                    agent_words.len()
                )));
            }
            if user_words.len() != 4 {
                return Err(FourWordError::InvalidInput(format!(
                    "User part must have 4 words, got {}",
                    user_words.len()
                )));
            }

            // Validate all words exist in dictionary
            for word in agent_words.iter().chain(user_words.iter()) {
                if DICTIONARY.get_index(word).is_none() {
                    return Err(FourWordError::InvalidWord(word.to_string()));
                }
            }

            Ok(IdentityWords::Full {
                agent_words: [
                    agent_words[0].to_lowercase(),
                    agent_words[1].to_lowercase(),
                    agent_words[2].to_lowercase(),
                    agent_words[3].to_lowercase(),
                ],
                user_words: [
                    user_words[0].to_lowercase(),
                    user_words[1].to_lowercase(),
                    user_words[2].to_lowercase(),
                    user_words[3].to_lowercase(),
                ],
            })
        } else {
            // Agent-only identity: 4 words
            let words: Vec<&str> = input.split_whitespace().collect();
            if words.len() != 4 {
                return Err(FourWordError::InvalidInput(format!(
                    "Agent identity must have 4 words, got {}",
                    words.len()
                )));
            }

            // Validate all words exist in dictionary
            for word in &words {
                if DICTIONARY.get_index(word).is_none() {
                    return Err(FourWordError::InvalidWord(word.to_string()));
                }
            }

            Ok(IdentityWords::Agent {
                words: [
                    words[0].to_lowercase(),
                    words[1].to_lowercase(),
                    words[2].to_lowercase(),
                    words[3].to_lowercase(),
                ],
            })
        }
    }

    /// Checks whether a 32-byte hash matches a set of identity words.
    ///
    /// Compares the first 48 bits of the hash against the prefix encoded
    /// by the words. Useful for verifying that an agent found via gossip
    /// actually matches the searched identity words.
    pub fn matches(&self, hash: &[u8], words: &str) -> Result<bool> {
        let prefix = self.decode_to_prefix(words)?;
        if hash.len() < 6 {
            return Ok(false);
        }
        Ok(hash[..6] == prefix[..])
    }
}

impl Default for IdentityEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real agent IDs observed on the x0x network (2026-04-03)
    const BEN_AGENT_ID: &str = "dd6530452610619d468e4e82be82107e86384365c58efa6e3018d7762c7368da";
    const DAVID_VPS_AGENT_ID: &str =
        "da2233d6ba2f95696e5f5ba3bc4db193be1aa53d7ce1c048a8e8a67639337b75";
    const THIRD_AGENT_ID: &str = "3e729de0469a594d1e042a672b29adde388e34aed2ced1e4c244a87f03053770";

    #[test]
    fn test_encode_agent_id() {
        let encoder = IdentityEncoder::new();
        let bytes = hex::decode(BEN_AGENT_ID).unwrap();
        let identity = encoder.encode_agent(&bytes).unwrap();

        assert!(matches!(identity, IdentityWords::Agent { .. }));
        assert_eq!(identity.word_count(), 4);

        let display = identity.to_string();
        let words: Vec<&str> = display.split_whitespace().collect();
        assert_eq!(words.len(), 4);

        // Each word should be in the dictionary
        for word in &words {
            assert!(
                DICTIONARY.get_index(word).is_some(),
                "Word '{}' not in dictionary",
                word
            );
        }

        println!("Ben's agent: {}", identity);
    }

    #[test]
    fn test_encode_all_network_agents() {
        let encoder = IdentityEncoder::new();

        let agents = [
            ("Ben", BEN_AGENT_ID),
            ("David VPS", DAVID_VPS_AGENT_ID),
            ("Third", THIRD_AGENT_ID),
        ];

        let mut seen = std::collections::HashSet::new();
        for (name, hex_id) in &agents {
            let identity = encoder.encode_hex(hex_id).unwrap();
            let display = identity.to_string();
            println!("{}: {} -> {}", name, &hex_id[..16], display);

            // No collisions between network agents
            assert!(
                seen.insert(display.clone()),
                "Collision detected for {}",
                name
            );
        }
    }

    #[test]
    fn test_round_trip_prefix() {
        let encoder = IdentityEncoder::new();
        let bytes = hex::decode(BEN_AGENT_ID).unwrap();

        let identity = encoder.encode_agent(&bytes).unwrap();
        let prefix = encoder.decode_to_prefix(&identity.to_string()).unwrap();

        // First 6 bytes (48 bits) should match exactly
        assert_eq!(&bytes[..6], &prefix[..]);
    }

    #[test]
    fn test_round_trip_all_agents() {
        let encoder = IdentityEncoder::new();

        for hex_id in [BEN_AGENT_ID, DAVID_VPS_AGENT_ID, THIRD_AGENT_ID] {
            let bytes = hex::decode(hex_id).unwrap();
            let identity = encoder.encode_agent(&bytes).unwrap();
            let prefix = encoder.decode_to_prefix(&identity.to_string()).unwrap();
            assert_eq!(
                &bytes[..6],
                &prefix[..],
                "Round-trip failed for {}",
                &hex_id[..16]
            );
        }
    }

    #[test]
    fn test_full_identity() {
        let encoder = IdentityEncoder::new();
        let full = encoder
            .encode_hex_full(BEN_AGENT_ID, THIRD_AGENT_ID)
            .unwrap();

        assert!(full.is_full());
        assert_eq!(full.word_count(), 8);

        let display = full.to_string();
        assert!(display.contains(" @ "), "Full identity must contain ' @ '");

        let parts: Vec<&str> = display.split(" @ ").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].split_whitespace().count(), 4);
        assert_eq!(parts[1].split_whitespace().count(), 4);

        println!("Full identity: {}", full);
    }

    #[test]
    fn test_parse_agent_identity() {
        let encoder = IdentityEncoder::new();
        let bytes = hex::decode(BEN_AGENT_ID).unwrap();

        // Encode then parse
        let identity = encoder.encode_agent(&bytes).unwrap();
        let display = identity.to_string();
        let parsed = encoder.parse(&display).unwrap();

        assert_eq!(identity, parsed);
    }

    #[test]
    fn test_parse_full_identity() {
        let encoder = IdentityEncoder::new();
        let full = encoder
            .encode_hex_full(BEN_AGENT_ID, THIRD_AGENT_ID)
            .unwrap();

        let display = full.to_string();
        let parsed = encoder.parse(&display).unwrap();

        assert_eq!(full, parsed);
    }

    #[test]
    fn test_matches() {
        let encoder = IdentityEncoder::new();
        let bytes = hex::decode(BEN_AGENT_ID).unwrap();

        let identity = encoder.encode_agent(&bytes).unwrap();
        let display = identity.to_string();

        // Should match the original hash
        assert!(encoder.matches(&bytes, &display).unwrap());

        // Should not match a different hash
        let other_bytes = hex::decode(DAVID_VPS_AGENT_ID).unwrap();
        assert!(!encoder.matches(&other_bytes, &display).unwrap());
    }

    #[test]
    fn test_different_agents_different_words() {
        let encoder = IdentityEncoder::new();

        let ben = encoder.encode_hex(BEN_AGENT_ID).unwrap().to_string();
        let david = encoder.encode_hex(DAVID_VPS_AGENT_ID).unwrap().to_string();
        let third = encoder.encode_hex(THIRD_AGENT_ID).unwrap().to_string();

        assert_ne!(ben, david);
        assert_ne!(ben, third);
        assert_ne!(david, third);
    }

    #[test]
    fn test_deterministic() {
        let encoder = IdentityEncoder::new();

        // Same input always produces same output
        let a = encoder.encode_hex(BEN_AGENT_ID).unwrap().to_string();
        let b = encoder.encode_hex(BEN_AGENT_ID).unwrap().to_string();
        assert_eq!(a, b);
    }

    #[test]
    fn test_family_name_pattern() {
        let encoder = IdentityEncoder::new();

        // Two different agents belonging to the same user
        // should share the last 4 words (the user's words)
        let full1 = encoder
            .encode_hex_full(BEN_AGENT_ID, THIRD_AGENT_ID)
            .unwrap();
        let full2 = encoder
            .encode_hex_full(DAVID_VPS_AGENT_ID, THIRD_AGENT_ID)
            .unwrap();

        // Agent words should differ (different agents)
        assert_ne!(full1.agent_words(), full2.agent_words());

        // User words should be identical (same user)
        assert_eq!(full1.user_words(), full2.user_words());

        println!("Agent 1: {}", full1);
        println!("Agent 2: {}", full2);
        println!(
            "Same family name: {}",
            full1.user_words().unwrap().join(" ")
        );
    }

    #[test]
    fn test_short_hash_rejected() {
        let encoder = IdentityEncoder::new();
        let short = vec![0u8; 5]; // Only 5 bytes, need 6
        assert!(encoder.encode_agent(&short).is_err());
    }

    #[test]
    fn test_invalid_word_rejected() {
        let encoder = IdentityEncoder::new();
        assert!(encoder.parse("not real words here").is_err());
    }

    #[test]
    fn test_wrong_word_count_rejected() {
        let encoder = IdentityEncoder::new();
        // Get a valid word to use
        let word = DICTIONARY.get_word(0).unwrap();
        let three_words = format!("{} {} {}", word, word, word);
        assert!(encoder.parse(&three_words).is_err());
    }
}
