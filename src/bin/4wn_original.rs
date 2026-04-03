//! 4wn - Four-Word Networking CLI
//!
//! Command-line tool with interactive autocomplete mode by default.
//!
//! Default (no args): Interactive mode with autocomplete hints and progressive completion
//! With arguments: Direct conversion between IP addresses and words
//!
//! Features:
//! - Interactive mode with hints at 3+ characters, auto-complete at 5 characters
//! - Perfect reconstruction for IPv4 (4 words) and adaptive compression for IPv6 (6/9/12 words)
//! - Real-time validation and completion suggestions
//!
//! Usage:
//!   4wn                         # Interactive mode (default)
//!   4wn 192.168.1.1:80          # Direct encode to 4 words
//!   4wn a abaddon amphipoda arian  # Direct decode to IP:port
//!   4wn [2001:db8::1]:443      # Direct encode IPv6 to words

use clap::Parser;
use four_word_networking::{FourWordAdaptiveEncoder, FourWordError, Result};
use std::io::{self, Write};
use std::process;

// TUI imports for real-time interactive mode
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};

#[derive(Parser)]
#[command(
    name = "4wn",
    about = "Four-Word Networking - Interactive IP address to words converter",
    long_about = "Interactive CLI for converting between IP addresses and memorable words.\n\
                  Default: Interactive mode with autocomplete hints and progressive completion.\n\
                  With arguments: Direct conversion between IP addresses and four-word combinations.\n\
                  Features perfect reconstruction for IPv4 (4 words) and adaptive compression for IPv6 (6/9/12 words).",
    version
)]
struct Cli {
    /// Input to convert (IP:port or words) - if provided, performs direct conversion
    /// If no input provided, starts interactive mode with autocomplete
    input: Vec<String>,

    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,

    /// Output format for scripting (minimal output)
    #[arg(short, long)]
    quiet: bool,

    /// Show completion hints for a prefix (utility mode)
    #[arg(short, long)]
    complete: Option<String>,

    /// Validate partial word input and show suggestions (utility mode)
    #[arg(long)]
    validate: Option<String>,
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

    // Handle utility modes first
    if let Some(prefix) = cli.complete {
        show_completion_hints(&encoder, &prefix)?;
        return Ok(());
    }

    if let Some(partial) = cli.validate {
        show_validation_results(&encoder, &partial)?;
        return Ok(());
    }

    // If no input provided, start interactive mode
    if cli.input.is_empty() {
        return interactive_mode(&encoder, cli.verbose);
    }

    // Direct conversion mode with arguments
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

/// Interactive mode with autocomplete and hints
/// Interactive mode with real-time character input and autocomplete
fn interactive_mode(encoder: &FourWordAdaptiveEncoder, verbose: bool) -> Result<()> {
    // Enable raw mode for character-by-character input
    terminal::enable_raw_mode()
        .map_err(|e| FourWordError::InvalidInput(format!("Failed to enable raw mode: {e}")))?;

    // Clean up terminal on exit
    let cleanup = || {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, ResetColor);
    };

    // Set up panic hook to cleanup terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, ResetColor);
        original_hook(info);
    }));

    let result = run_interactive_tui(encoder, verbose);

    // Always cleanup
    cleanup();

    result
}

/// Run the actual TUI interactive mode
fn run_interactive_tui(encoder: &FourWordAdaptiveEncoder, verbose: bool) -> Result<()> {
    let mut stdout = io::stdout();

    // Clear screen and show header
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(
            "🌐 Four-Word Networking - Interactive Mode
"
        ),
        Print(
            "Real-time autocomplete: Progressive hints at 3+ chars, auto-complete at 5 chars
"
        ),
        Print(
            "Commands: quit/exit to leave, Ctrl+C to interrupt, Tab for completion

"
        )
    )
    .map_err(|e| FourWordError::InvalidInput(format!("Terminal error: {e}")))?;

    let mut current_input = String::new();
    let mut cursor_pos = 0;
    let mut completed_words = Vec::<String>::new();

    loop {
        // Render current state
        render_ui(
            &mut stdout,
            &current_input,
            cursor_pos,
            &completed_words,
            encoder,
            verbose,
        )?;

        // Read next event
        if let Ok(event) = event::read() {
            match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('c')
                            if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char('q')
                            if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Enter => {
                            if handle_enter(&current_input, &mut completed_words, encoder, verbose)?
                            {
                                break;
                            }
                            current_input.clear();
                            cursor_pos = 0;
                        }
                        KeyCode::Tab => {
                            // Handle tab completion
                            if let Some(completion) = get_best_completion(encoder, &current_input) {
                                current_input = completion;
                                cursor_pos = current_input.len();
                            }
                        }
                        KeyCode::Backspace => {
                            if cursor_pos > 0 {
                                current_input.remove(cursor_pos - 1);
                                cursor_pos -= 1;
                            }
                        }
                        KeyCode::Delete => {
                            if cursor_pos < current_input.len() {
                                current_input.remove(cursor_pos);
                            }
                        }
                        KeyCode::Left => {
                            cursor_pos = cursor_pos.saturating_sub(1);
                        }
                        KeyCode::Right => {
                            if cursor_pos < current_input.len() {
                                cursor_pos += 1;
                            }
                        }
                        KeyCode::Home => {
                            cursor_pos = 0;
                        }
                        KeyCode::End => {
                            cursor_pos = current_input.len();
                        }
                        KeyCode::Char(c) => {
                            // Insert character at cursor position
                            current_input.insert(cursor_pos, c);
                            cursor_pos += 1;

                            // Auto-complete at 5 characters if unique
                            if current_input.len() >= 5
                                && let Some(word) = encoder.auto_complete_at_five(&current_input)
                            {
                                current_input = word;
                                cursor_pos = current_input.len();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Clear screen and show goodbye
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(
            "Goodbye! 👋
"
        )
    )
    .map_err(|e| FourWordError::InvalidInput(format!("Terminal error: {e}")))?;

    Ok(())
}

/// Render the TUI interface
fn render_ui(
    stdout: &mut io::Stdout,
    current_input: &str,
    cursor_pos: usize,
    completed_words: &[String],
    encoder: &FourWordAdaptiveEncoder,
    _verbose: bool,
) -> Result<()> {
    // Move to start of input area (line 5)
    queue!(stdout, cursor::MoveTo(0, 4))?;

    // Clear from cursor to end of screen
    queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;

    // Show completed words
    if !completed_words.is_empty() {
        queue!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Words: "),
            ResetColor,
            Print(&completed_words.join(" ")),
            Print(&format!(" ({}/4)", completed_words.len())),
            Print(
                "

"
            )
        )?;
    }

    // Show current input prompt
    queue!(stdout, Print("4wn> "), Print(current_input))?;

    // Show hints if input is 3+ characters
    if current_input.len() >= 3 {
        let hints = encoder.get_word_hints(current_input);
        if !hints.is_empty() {
            queue!(
                stdout,
                Print(
                    "

"
                )
            )?;

            if hints.len() == 1 {
                queue!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("✓ Complete match: "),
                    ResetColor,
                    Print(&hints[0])
                )?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    Print(&format!("💡 {} matches: ", hints.len())),
                    ResetColor
                )?;

                // Show up to 5 hints
                for (i, hint) in hints.iter().take(5).enumerate() {
                    if i > 0 {
                        queue!(stdout, Print(", "))?;
                    }
                    queue!(stdout, Print(hint))?;
                }
                if hints.len() > 5 {
                    queue!(stdout, Print(&format!(" (+{} more)", hints.len() - 5)))?;
                }
            }

            // Show auto-complete message at 5+ chars
            if current_input.len() >= 5 && hints.len() == 1 {
                queue!(
                    stdout,
                    Print(
                        "
"
                    ),
                    SetForegroundColor(Color::Cyan),
                    Print("   Press any key to auto-complete"),
                    ResetColor
                )?;
            }
        } else {
            queue!(
                stdout,
                Print(
                    "

"
                ),
                SetForegroundColor(Color::Red),
                Print("❌ No matches found"),
                ResetColor
            )?;
        }
    }

    // Position cursor at input location
    let cursor_col = 5 + cursor_pos; // "4wn> " = 5 chars
    queue!(
        stdout,
        cursor::MoveTo(
            cursor_col as u16,
            4 + if completed_words.is_empty() { 0 } else { 2 }
        )
    )?;

    stdout
        .flush()
        .map_err(|e| FourWordError::InvalidInput(format!("Flush error: {e}")))?;
    Ok(())
}

/// Handle Enter key press
fn handle_enter(
    input: &str,
    completed_words: &mut Vec<String>,
    encoder: &FourWordAdaptiveEncoder,
    _verbose: bool,
) -> Result<bool> {
    let input = input.trim();

    // Handle special commands
    match input.to_lowercase().as_str() {
        "quit" | "exit" => return Ok(true),
        "clear" => {
            completed_words.clear();
            return Ok(false);
        }
        "" => return Ok(false),
        _ => {}
    }

    // Try to process as complete address
    if (input.contains(':') || input.contains('[') || input.parse::<std::net::IpAddr>().is_ok())
        && let Ok(encoded) = encoder.encode(input)
    {
        println!("\n🌐 {input} → {encoded}\n");
        return Ok(false);
    }

    // Try to process as complete word sequence
    if looks_like_words(input)
        && let Ok(decoded) = encoder.decode(input)
    {
        println!("\n🌐 {input} → {decoded}\n");
        return Ok(false);
    }

    // Add as word to completion
    if encoder.is_valid_prefix(input) {
        let hints = encoder.get_word_hints(input);
        if hints.len() == 1 {
            completed_words.push(hints[0].clone());

            // Check if we have 4 words (complete IPv4)
            if completed_words.len() == 4 {
                let word_str = completed_words.join(" ");
                match encoder.decode(&word_str) {
                    Ok(decoded) => {
                        println!(
                            "
✅ Complete! {word_str} → {decoded}
"
                        );
                        completed_words.clear();
                    }
                    Err(_) => {
                        println!(
                            "
❌ Invalid word combination
"
                        );
                        completed_words.clear();
                    }
                }
            }
        } else {
            println!(
                "
❌ Ambiguous input: {} matches found
",
                hints.len()
            );
        }
    } else {
        println!(
            "
❌ Invalid word or prefix
"
        );
    }

    Ok(false)
}

/// Get the best completion for current input
fn get_best_completion(encoder: &FourWordAdaptiveEncoder, input: &str) -> Option<String> {
    if input.len() >= 3 {
        let hints = encoder.get_word_hints(input);
        if hints.len() == 1 {
            Some(hints[0].clone())
        } else if !hints.is_empty() {
            // Return the shortest match
            hints.into_iter().min_by_key(|s| s.len())
        } else {
            None
        }
    } else {
        None
    }
}

/// Process input as complete IP address or word sequence
#[allow(dead_code)]
fn process_complete_input(
    encoder: &FourWordAdaptiveEncoder,
    input: &str,
    verbose: bool,
) -> Result<Option<String>> {
    // Check if it looks like an IP address or complete word sequence
    if input.contains(':') || input.contains('[') || input.parse::<std::net::IpAddr>().is_ok() {
        // Looks like IP address
        let encoded = encoder.encode(input)?;
        if verbose {
            Ok(Some(format!("{input} → {encoded}")))
        } else {
            Ok(Some(encoded))
        }
    } else if looks_like_words(input) {
        // Looks like complete word sequence
        let decoded = encoder.decode(input)?;
        if verbose {
            Ok(Some(format!("{input} → {decoded}")))
        } else {
            Ok(Some(decoded))
        }
    } else {
        // Not complete input
        Err(FourWordError::InvalidInput(
            "Not complete input".to_string(),
        ))
    }
}

/// Handle progressive input with autocomplete
#[allow(dead_code)]
fn handle_progressive_input(
    encoder: &FourWordAdaptiveEncoder,
    input: &str,
    current_input: &mut String,
    current_words: &mut Vec<String>,
) {
    current_input.push_str(input);

    // Check for space (word completion)
    if current_input.contains(' ') {
        let parts: Vec<&str> = current_input.split(' ').collect();
        if let Some(word) = parts.first().filter(|w| !w.is_empty()) {
            // Complete the current word
            if let Some(completed) = try_complete_word(encoder, word) {
                current_words.push(completed);
                *current_input = parts[1..].join(" ");

                // Check if we have 4 words (complete IPv4)
                if current_words.len() == 4 {
                    let word_sequence = current_words.join(" ");
                    if let Ok(decoded) = encoder.decode(&word_sequence) {
                        println!("✅ Complete! {word_sequence} → {decoded}");
                    }
                    current_words.clear();
                    current_input.clear();
                }
            } else {
                println!("❌ '{word}' is not a valid word. Try again.");
                current_input.clear();
            }
        }
    } else {
        // Show hints for current input
        show_progressive_hints(encoder, current_input);
    }
}

/// Try to complete a partial word
#[allow(dead_code)]
fn try_complete_word(encoder: &FourWordAdaptiveEncoder, partial: &str) -> Option<String> {
    // Auto-complete at 5 characters
    if partial.len() >= 5 {
        return encoder.auto_complete_at_five(partial);
    }

    // Check if it's already a complete word
    let hints = encoder.get_word_hints(partial);
    if hints.len() == 1 && hints[0] == partial {
        return Some(partial.to_string());
    }

    None
}

/// Show progressive hints for current input
#[allow(dead_code)]
fn show_progressive_hints(encoder: &FourWordAdaptiveEncoder, input: &str) {
    if input.len() < 3 {
        return;
    }

    let hints = encoder.get_word_hints(input);
    match hints.len() {
        0 => println!("❌ No words start with '{input}'"),
        1 => {
            if hints[0] == input {
                println!("✅ '{input}' is complete");
            } else {
                println!("💡 Complete: {}", hints[0]);
            }
        }
        2..=5 => {
            println!("💡 Hints: {}", hints.join(", "));
        }
        _ => {
            println!(
                "💡 {} possibilities: {}, ...",
                hints.len(),
                hints[..3].join(", ")
            );
        }
    }
}

/// Show completion hints utility function
fn show_completion_hints(encoder: &FourWordAdaptiveEncoder, prefix: &str) -> Result<()> {
    let hints = encoder.get_word_hints(prefix);
    if hints.is_empty() {
        println!("No words found starting with '{prefix}'");
    } else {
        println!("Completions for '{}' ({} found):", prefix, hints.len());
        for hint in hints.iter().take(10) {
            println!("  {hint}");
        }
        if hints.len() > 10 {
            println!("  ... and {} more", hints.len() - 10);
        }
    }
    Ok(())
}

/// Show validation results utility function
fn show_validation_results(encoder: &FourWordAdaptiveEncoder, partial: &str) -> Result<()> {
    let result = encoder.validate_partial_input(partial)?;

    println!("Validation results for '{partial}':");
    println!("  Valid prefix: {}", result.is_valid_prefix);
    println!("  Words so far: {}", result.word_count_so_far);
    println!("  Is complete: {}", result.is_complete);

    if !result.possible_completions.is_empty() {
        println!("  Completions: {}", result.possible_completions.join(", "));
    }

    Ok(())
}

/// Show help information
#[allow(dead_code)]
fn show_help() {
    println!("🌐 Four-Word Networking Help");
    println!();
    println!("Interactive Mode Commands:");
    println!("  help     - Show this help");
    println!("  clear    - Clear current input");
    println!("  quit     - Exit the program");
    println!();
    println!("Usage:");
    println!("  • Type IP address → get 4 words (IPv4) or 6/9/12 words (IPv6)");
    println!("  • Type words → get IP address");
    println!("  • Progressive hints appear at 3+ characters");
    println!("  • Auto-completion happens at 5+ characters");
    println!("  • Space completes current word and moves to next");
    println!();
    println!("Examples:");
    println!("  192.168.1.1    → four memorable words");
    println!("  about beam cat → reconstructed IP address");
    println!("  ::1            → six words for IPv6 loopback");
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
