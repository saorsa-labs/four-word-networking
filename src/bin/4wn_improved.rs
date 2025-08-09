//! 4wn - Four-Word Networking CLI with Improved Interactive Mode
//!
//! Much better UX with real-time conversion and intelligent input detection

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
    about = "Four-Word Networking - Smart IP/word converter",
    long_about = "Smart interactive CLI that auto-detects whether you're typing an IP address or words.\n\
                  Just start typing - it figures out what you mean and shows live conversion.",
    version
)]
struct Cli {
    /// Input to convert (IP:port or words) - if provided, performs direct conversion
    /// If no input provided, starts interactive mode
    input: Vec<String>,

    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,

    /// Output format for scripting (minimal output)
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum InputType {
    IpAddress,
    Words,
    Unknown,
    Command,
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

    // If no input provided, start interactive mode
    if cli.input.is_empty() {
        return interactive_mode(&encoder, cli.verbose);
    }

    // Direct conversion mode with arguments
    let input = if cli.input.len() == 1 {
        cli.input[0].trim().to_string()
    } else {
        // Multiple arguments - treat as space-separated words
        cli.input.join(" ")
    };

    // Detect and process
    if looks_like_ip(&input) {
        encode_address(&encoder, &input, cli.verbose, cli.quiet)
    } else {
        decode_words(&encoder, &input, cli.verbose, cli.quiet)
    }
}

/// Improved IP detection
fn looks_like_ip(input: &str) -> bool {
    // Check for IP patterns
    input.contains(':')
        || input.contains('[')
        || input.parse::<std::net::IpAddr>().is_ok()
        || input.chars().filter(|c| *c == '.').count() == 3 && input.chars().any(|c| c.is_numeric())
}

/// Detect input type intelligently
fn detect_input_type(input: &str) -> InputType {
    let trimmed = input.trim();

    // Check for commands first
    if trimmed == "quit" || trimmed == "exit" || trimmed == "help" || trimmed == "clear" {
        return InputType::Command;
    }

    // Check if it's starting to look like an IP
    if input.contains(':') || input.contains('[') {
        return InputType::IpAddress;
    }

    // Check for IPv4 pattern (numbers and dots)
    let has_numbers = input.chars().any(|c| c.is_numeric());
    let dot_count = input.chars().filter(|c| *c == '.').count();

    if has_numbers && (dot_count > 0 || input.parse::<u8>().is_ok()) {
        return InputType::IpAddress;
    }

    // Check if it's words (alphabetic with spaces or dots)
    let mostly_alpha = input
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace() || *c == '.' || *c == '-')
        .count()
        == input.len();

    if mostly_alpha && !input.is_empty() {
        return InputType::Words;
    }

    InputType::Unknown
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
        println!("{words}");
    } else if verbose {
        println!("Input: {address}");
        println!("Words: {words}");
        println!(
            "Type: {}",
            if words.contains('-') { "IPv6" } else { "IPv4" }
        );
    } else {
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
        println!("{address}");
    } else if verbose {
        println!("Input: {words}");
        println!("Address: {address}");
    } else {
        println!("{address}");
    }

    Ok(())
}

/// Improved interactive mode
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

/// Run the improved TUI
fn run_interactive_tui(encoder: &FourWordAdaptiveEncoder, _verbose: bool) -> Result<()> {
    let mut stdout = io::stdout();

    // Clear screen and show better header
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        SetForegroundColor(Color::Cyan),
        Print("🌐 Four-Word Networking\n"),
        ResetColor,
        Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n"),
        Print("Just start typing! I'll figure out if it's an IP or words.\n"),
        Print("• IP addresses → instant word conversion\n"),
        Print("• Words → instant IP reconstruction\n"),
        Print("• Tab completes partial words\n"),
        Print("• Type 'help' for more commands\n\n"),
    )
    .map_err(|e| FourWordError::InvalidInput(format!("Terminal error: {e}")))?;

    let mut current_input = String::new();
    let mut cursor_pos = 0;
    let mut _last_result = String::new();

    loop {
        // Detect what the user is typing
        let input_type = detect_input_type(&current_input);

        // Try live conversion
        let live_result = try_live_conversion(encoder, &current_input, input_type.clone());

        // Render the UI
        render_smart_ui(
            &mut stdout,
            &current_input,
            cursor_pos,
            &input_type,
            &live_result,
            encoder,
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
                        KeyCode::Enter => {
                            // Process the input
                            let result = process_input(encoder, &current_input);
                            match result {
                                Ok(Some(output)) => {
                                    _last_result = output;
                                    if current_input.trim() == "quit"
                                        || current_input.trim() == "exit"
                                    {
                                        break;
                                    }
                                }
                                Ok(None) => {
                                    // Command handled
                                }
                                Err(e) => {
                                    _last_result = format!("Error: {e}");
                                }
                            }
                            current_input.clear();
                            cursor_pos = 0;
                        }
                        KeyCode::Tab => {
                            // Smart tab completion
                            if let Some(completed) =
                                smart_complete(encoder, &current_input, cursor_pos)
                            {
                                current_input = completed;
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
                            current_input.insert(cursor_pos, c);
                            cursor_pos += 1;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Clear and say goodbye
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print("Thanks for using 4wn! 👋\n")
    )
    .map_err(|e| FourWordError::InvalidInput(format!("Terminal error: {e}")))?;

    Ok(())
}

/// Smart UI rendering
fn render_smart_ui(
    stdout: &mut io::Stdout,
    input: &str,
    cursor_pos: usize,
    input_type: &InputType,
    live_result: &Option<String>,
    encoder: &FourWordAdaptiveEncoder,
) -> Result<()> {
    // Move to input line (line 10)
    queue!(stdout, cursor::MoveTo(0, 9))?;
    queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;

    // Show input type detection
    match input_type {
        InputType::IpAddress => {
            queue!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print("📍 IP Address Mode\n"),
                ResetColor
            )?;
        }
        InputType::Words => {
            queue!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("📝 Word Mode\n"),
                ResetColor
            )?;
        }
        InputType::Command => {
            queue!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print("⚡ Command\n"),
                ResetColor
            )?;
        }
        InputType::Unknown => {
            queue!(
                stdout,
                SetForegroundColor(Color::Grey),
                Print("💭 Type an IP or words...\n"),
                ResetColor
            )?;
        }
    }

    // Show the input prompt
    queue!(stdout, Print("\n> "), Print(input))?;

    // Show live conversion result
    if let Some(result) = live_result {
        queue!(
            stdout,
            Print("\n\n"),
            SetForegroundColor(Color::Cyan),
            Print("→ "),
            ResetColor,
            Print(result)
        )?;
    }

    // Show hints for words mode
    if matches!(input_type, InputType::Words) && !input.is_empty() {
        // Get the last partial word
        let words: Vec<&str> = input.split_whitespace().collect();
        if let Some(last_word) = words.last() {
            if !last_word.is_empty() {
                let hints = encoder.get_word_hints(last_word);
                if !hints.is_empty() && hints.len() <= 10 {
                    queue!(stdout, Print("\n\nHints: "))?;
                    for (i, hint) in hints.iter().take(5).enumerate() {
                        if i > 0 {
                            queue!(stdout, Print(", "))?;
                        }
                        queue!(
                            stdout,
                            SetForegroundColor(Color::DarkGrey),
                            Print(hint),
                            ResetColor
                        )?;
                    }
                }
            }
        }
    }

    // Position cursor correctly
    let cursor_col = 2 + cursor_pos; // "> " = 2 chars
    queue!(stdout, cursor::MoveTo(cursor_col as u16, 11))?;

    stdout
        .flush()
        .map_err(|e| FourWordError::InvalidInput(format!("Flush error: {e}")))?;
    Ok(())
}

/// Try live conversion as user types
fn try_live_conversion(
    encoder: &FourWordAdaptiveEncoder,
    input: &str,
    input_type: InputType,
) -> Option<String> {
    if input.trim().is_empty() {
        return None;
    }

    match input_type {
        InputType::IpAddress => {
            // Try to encode partial or complete IP
            encoder.encode(input).ok()
        }
        InputType::Words => {
            // Try to decode if we have complete words
            let words: Vec<&str> = input.split_whitespace().collect();
            if words.len() == 4 || words.len() == 6 || words.len() == 9 || words.len() == 12 {
                encoder.decode(input).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Smart completion
fn smart_complete(
    encoder: &FourWordAdaptiveEncoder,
    input: &str,
    _cursor_pos: usize,
) -> Option<String> {
    // Find the word at cursor position
    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return None;
    }

    // Get the last word (partial)
    if let Some(last_word) = words.last() {
        if !last_word.is_empty() {
            let hints = encoder.get_word_hints(last_word);
            if hints.len() == 1 {
                // Complete with the single match
                let mut result = words[..words.len() - 1].join(" ");
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(&hints[0]);
                return Some(result);
            } else if !hints.is_empty() {
                // Use the shortest match
                if let Some(shortest) = hints.iter().min_by_key(|s| s.len()) {
                    let mut result = words[..words.len() - 1].join(" ");
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(shortest);
                    return Some(result);
                }
            }
        }
    }

    None
}

/// Process the completed input
fn process_input(encoder: &FourWordAdaptiveEncoder, input: &str) -> Result<Option<String>> {
    let trimmed = input.trim();

    // Handle commands
    match trimmed {
        "quit" | "exit" => return Ok(Some("quit".to_string())),
        "help" => {
            return Ok(Some(
                "Commands:\n\
                • Type any IP address to see its words\n\
                • Type words to see the IP address\n\
                • Tab - complete partial word\n\
                • clear - clear the screen\n\
                • quit/exit - leave the program"
                    .to_string(),
            ));
        }
        "clear" => {
            let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
            return Ok(None);
        }
        "" => return Ok(None),
        _ => {}
    }

    // Try as IP first
    if let Ok(encoded) = encoder.encode(trimmed) {
        return Ok(Some(format!("{trimmed} → {encoded}")));
    }

    // Try as words
    if let Ok(decoded) = encoder.decode(trimmed) {
        return Ok(Some(format!("{trimmed} → {decoded}")));
    }

    // Check if it's partial words that need completion
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if !words.is_empty() {
        // Try to validate each word
        for word in &words {
            if encoder.get_word_hints(word).is_empty()
                && !encoder.get_word_hints(word).contains(&word.to_string())
            {
                return Err(FourWordError::InvalidInput(format!(
                    "'{word}' is not a valid word"
                )));
            }
        }
    }

    Err(FourWordError::InvalidInput("Invalid input".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_type_detection() {
        assert_eq!(detect_input_type("192.168"), InputType::IpAddress);
        assert_eq!(detect_input_type("ocean blue"), InputType::Words);
        assert_eq!(detect_input_type("quit"), InputType::Command);
        assert_eq!(detect_input_type(""), InputType::Unknown);
        assert_eq!(detect_input_type("127.0"), InputType::IpAddress);
        assert_eq!(detect_input_type("::1"), InputType::IpAddress);
        assert_eq!(detect_input_type("a"), InputType::Words);
    }
}
