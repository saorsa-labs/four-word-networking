//! 4wn - Four-Word Networking CLI
//!
//! Simple command-line tool that automatically detects whether input is:
//! - 4 words (IPv4) or 8/12 words (IPv6) → decode to IP:port
//! - IP:port → encode to 4 words (IPv4) or 8/12 words (IPv6)
//!
//! Features 100% perfect reconstruction for IPv4 and adaptive compression for IPv6.
//!
//! Usage:
//!   4wn 192.168.1.1:80          # Encodes to 4 words (perfect)
//!   4wn a abaddon amphipoda arian  # Decodes to exact IPv4:port
//!   4wn [2001:db8::1]:443      # Encodes to 8 or 12 words with visual distinction
//!   4wn ocean thunder falcon star book april wing moon    # Decodes to IPv6

use clap::Parser;
use four_word_networking::{FourWordAdaptiveEncoder, Result};
use std::process;

#[derive(Parser)]
#[command(
    name = "4wn",
    about = "Four-Word Networking - Convert between IP addresses and memorable words",
    long_about = "Automatically converts between IP addresses and four-word combinations.\n\
                  Features 100% perfect reconstruction for IPv4 and adaptive compression for IPv6.\n\
                  IPv4 uses 4 words with spaces, IPv6 uses 8 or 12 words with spaces.",
    version
)]
struct Cli {
    /// Input to convert (IP:port or words)
    /// Can be a single string or multiple words
    input: Vec<String>,

    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,

    /// Output format for scripting (minimal output)
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let encoder = FourWordAdaptiveEncoder::new()?;

    // Join input arguments
    let input = if cli.input.len() == 1 {
        // Single argument - could be IP or words with separators
        cli.input[0].trim().to_string()
    } else {
        // Multiple arguments - treat as space-separated words
        cli.input.join(" ")
    };

    // Detect input type based on content
    if looks_like_words(&input) {
        // Input is words, decode to IP:port
        decode_words(&encoder, &input, cli.verbose, cli.quiet)
    } else {
        // Input is IP:port, encode to words
        encode_address(&encoder, &input, cli.verbose, cli.quiet)
    }
}

/// Check if input looks like words (contains dots, dashes, spaces, all alphabetic)
fn looks_like_words(input: &str) -> bool {
    // Handle space-separated words or separator-based words
    let segments: Vec<&str> = if input.contains(' ') && !input.contains('-') && !input.contains(':')
    {
        // Space-separated words (not IPv6 with colons)
        input.split_whitespace().collect()
    } else if input.contains('.')
        || input.contains('-')
        || input.contains('_')
        || input.contains('+')
    {
        // Split by any separator
        input.split(|c: char| ".-_+".contains(c)).collect()
    } else {
        // No separators - not words
        return false;
    };

    // Must be 4 (IPv4), 6, 9, or 12 (IPv6) segments
    if segments.len() != 4 && segments.len() != 6 && segments.len() != 9 && segments.len() != 12 {
        return false;
    }

    // Check if all segments are alphabetic and meet minimum length requirement
    segments
        .iter()
        .all(|segment| !segment.is_empty() && segment.chars().all(|c| c.is_alphabetic()))
}

/// Encode IP address to words
fn encode_address(
    encoder: &FourWordAdaptiveEncoder,
    address: &str,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    let words = encoder.encode(address)?;

    if quiet {
        // Minimal output for scripting
        println!("{words}");
    } else if verbose {
        // Detailed output
        println!("Input: {address}");
        println!("Words: {words}");
        println!("Encoding: Perfect (100% reversible)");

        if words.contains('.') && !words.contains('-') {
            println!("Type: IPv4 (dot separators, lowercase)");
        } else if words.contains('-') {
            println!("Type: IPv6 (dash separators, title case)");
        }

        println!("Features:");
        println!("  • Perfect IPv4 reconstruction (4 words)");
        println!("  • Adaptive IPv6 compression (6, 9, or 12 words)");
        println!("  • Guaranteed perfect reconstruction");
    } else {
        // Normal output
        println!("{words}");
    }

    Ok(())
}

/// Decode words to IP address
fn decode_words(
    encoder: &FourWordAdaptiveEncoder,
    words: &str,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    let address = encoder.decode(words)?;

    if quiet {
        // Minimal output for scripting
        println!("{address}");
    } else if verbose {
        // Detailed output
        println!("Input: {words}");
        println!("Address: {address}");
        println!("Decoding: Perfect reconstruction");

        if words.contains('.') && !words.contains('-') {
            println!("Type: IPv4 (detected from dot separators)");
        } else if words.contains('-') {
            println!("Type: IPv6 (detected from dash separators)");
        }
    } else {
        // Normal output
        println!("{address}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_words() {
        // Valid words - 4 words with spaces
        assert!(looks_like_words("ocean thunder falcon star"));

        // Valid words - 6 words with spaces
        assert!(looks_like_words("ocean thunder falcon star book april"));

        // Valid words - 9 words with spaces
        assert!(looks_like_words(
            "ocean thunder falcon star book april wing moon river"
        ));

        // Valid words - 4 words with dots
        assert!(looks_like_words("ocean.thunder.falcon.star"));

        // Invalid - wrong count
        assert!(!looks_like_words("ocean.thunder.falcon"));
        assert!(!looks_like_words("a.b.c.d.e"));

        // Invalid - contains non-alphabetic
        assert!(!looks_like_words("ocean.thunder.123"));
        assert!(!looks_like_words("192.168.1.1"));
        assert!(!looks_like_words("ocean:thunder:falcon"));
    }
}
