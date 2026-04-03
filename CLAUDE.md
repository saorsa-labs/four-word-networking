# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Four-Word Networking is a Rust library and CLI that converts IP addresses into memorable word combinations for human-friendly networking. The system provides perfect reconstruction for IPv4 addresses using exactly 4 words, and adaptive compression for IPv6 addresses using 6, 9, or 12 words (groups of 4) with intelligent category-based optimization.

## Common Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Build with release optimizations
cargo build --release

# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_four_word_address_parsing

# Run main test suite
./run_main_tests.sh

# Run the CLI
cargo run --bin 4wn -- 192.168.1.1:443
```

### Code Quality
```bash
# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy for linting
cargo clippy -- -D warnings

# Check for unused dependencies
cargo machete
```

### CLI Usage Examples
```bash
# Convert IPv4 to four words (perfect reconstruction)
cargo run --bin 4wn -- 192.168.1.1:443

# Convert IPv6 to 6 or 9 words (groups of 3)
cargo run --bin 4wn -- "[::1]:443"

# Decode words back to IP addresses (dots or spaces)
cargo run --bin 4wn -- beatniks.contrarily.stockholm
cargo run --bin 4wn -- beatniks contrarily stockholm

# Decode IPv6 from words
cargo run --bin 4wn -- sectorial supper ballparks consider tri gram

# Verbose output
cargo run --bin 4wn -- -v 192.168.1.1:443
```

### Binary Tools
```bash
# Build and validate the dictionary systems
cargo run --bin validate_16k_system

# Check word quality in dictionary
cargo run --bin check_word_quality

# Create clean dictionary (removes homophones, offensive words)
cargo run --bin create_clean_dictionary

# Test four-word system
cargo run --bin test_four_word

# Benchmark three vs four word systems
cargo run --bin benchmark_three_vs_four
```

## Architecture

### Core Components

- **`src/lib.rs`**: Main library interface and public API
- **`src/four_word_adaptive_encoder.rs`**: Four-word adaptive encoder system for perfect IPv4 and adaptive IPv6
- **`src/ipv6_compression.rs`**: IPv6 category-based compression algorithms
- **`src/error.rs`**: Comprehensive error types using `thiserror`
- **`src/main.rs`**: CLI application using `clap`
- **`src/bin/4wn.rs`**: Command-line interface for four-word networking

### Four-Word Encoding Systems

- **`src/dictionary4k.rs`**: 4,096-word dictionary for IPv4 4-word encoding (2^12 words)
- **`src/four_word_encoder.rs`**: Perfect 4-word encoding for IPv4 with Feistel network
- **`src/four_word_ipv6_encoder.rs`**: Groups of 4 encoding for IPv6 (6, 9, or 12 words)
- **`src/four_word_adaptive_encoder.rs`**: Main interface for encoding/decoding IP addresses

### Advanced Encoding Systems

- **`src/compression.rs`**: IP address compression achieving 40-60% reduction
- **`src/ipv6_compression.rs`**: Category-based IPv6 compression
- **`src/encoder16k.rs`**: Legacy four-word encoder using 14-bit word indices
- **`src/balanced_encoder.rs`**: Natural word grouping with compression
- **`src/ultra_compression.rs`**: Aggressive compression for ≤5 byte output

### Key Data Structures

- **`FourWordEncoding`**: Four-word IPv4 address structure
- **`FourWordAdaptiveEncoder`**: Main interface for encoding/decoding IP addresses
- **`Dictionary4K`**: 4,096-word dictionary for 12-bit per word encoding
- **`Ipv6FourWordGroupEncoding`**: IPv6 encoding in groups of 4 words
- **`FourWordGroup`**: Container for 4-word groups
- **`CompressedIpv6`**: IPv6 compression with category-based optimization
- **`Ipv6Category`**: IPv6 address types (Loopback, LinkLocal, GlobalUnicast, etc.)

## Encoding Strategies

### Four-Word IPv4 Encoding
- **Perfect Reconstruction**: 48 bits (IPv4 + port) encoded in 4 × 12-bit words
- **Feistel Network**: 8 rounds of cryptographic bit diffusion
- **Dictionary**: 4,096 words (2^12) for perfect 12-bit encoding
- **Format**: Lowercase words separated by dots or spaces

### IPv6 Group Encoding
- **Groups of 4**: Always 6, 9, or 12 words (variable groups)
- **Category Detection**: Optimizes based on IPv6 type
- **Compression**: 6 words for common patterns, 9 for medium complexity, 12 for complex addresses
- **Format**: Title case words separated by dashes

### Compression Techniques
```rust
// Example: IPv6 category-based compression
match category {
    Ipv6Category::Loopback => compress_loopback(),
    Ipv6Category::LinkLocal => compress_link_local(),
    Ipv6Category::Documentation => compress_documentation(),
    Ipv6Category::GlobalUnicast => compress_global(),
}
```

## Dictionary Management

### 4K Dictionary (IPv4)
- **Size**: 4,096 words (2^12)
- **Sources**: EFF, BIP39, Diceware, custom English words
- **Quality**: Voice-friendly, no homophones, 3-7 characters preferred

### Word Quality Criteria
- Length: 3-7 characters optimal
- Voice-friendly: Easy to pronounce
- No homophones or offensive terms
- Phonetically distinct
- Common English usage preferred

## Development Patterns

### Error Handling
```rust
// Always use Result types
pub fn encode(addr: &str) -> Result<String, FourWordError> {
    // Implementation
}

// Use ? operator for propagation
let parsed = parse_address(addr)?;
```

### Testing Strategy
- Unit tests in `#[cfg(test)]` modules
- Integration tests for workflows
- Real-world address testing
- Performance benchmarks (<2μs requirement)
- CLI integration tests

### Code Organization
- Feature-focused module structure
- Clear separation of concerns
- Comprehensive rustdoc documentation
- Examples in all public APIs

## Performance Targets

- **Encoding**: <1μs for IPv4, <2μs for IPv6
- **Decoding**: <1μs for IPv4, <2μs for IPv6
- **Memory**: ~1MB total dictionary size
- **Throughput**: ~1,000,000 addresses/second
- **Zero Collisions**: Deterministic encoding

## Current Implementation Status

### Production Ready
- Four-word IPv4 encoding with perfect reconstruction
- IPv6 encoding in groups of 3 (6 or 9 or 12 words)
- 4K word dictionary system
- CLI with full feature set (`3wn`)
- Space-separated word support
- Comprehensive test coverage

### Features
- IPv4: Always exactly 4 words
- IPv6: 6 words for common patterns, 9 for complex
- Visual distinction: dots vs dashes, case differences
- Voice-optimized dictionary
- Sub-microsecond performance

### Known Limitations
- English-only dictionaries currently
- Some IPv6 patterns may require 9 words

## Future Development Areas

### High Priority
- Multi-language dictionary support
- WebAssembly bindings
- Python/JavaScript bindings
- Integration with networking libraries

### Medium Priority
- GUI applications
- Browser extensions
- Mobile SDKs
- Network visualization tools

## Dependencies

### Core
- `serde`: Serialization (with derive)
- `thiserror`: Error handling
- `clap`: CLI parsing (with derive)
- `once_cell`: Global dictionary singleton

### Testing
- `tokio-test`: Async test utilities
- `rand`: Random generation
- `criterion`: Benchmarking

## Useful Resources

- **Test Script**: `./run_main_tests.sh` for test suite
- **Binary Tools**: Multiple utilities in `src/bin/` for development
- **Word Lists**: Raw dictionaries in `wordlists/` directory
- **4K Dictionary**: Pre-built in `data/dictionary_4096.txt`

## Code Quality Standards

### Before every commit, run:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Clippy
- Must pass with ZERO warnings
- Run with `--all-targets` (not just lib) to catch issues in tests and bins
- Run with `-- -D warnings` to treat all warnings as errors
- Fix collapsible if statements — use `if cond && let Ok(x) = expr` (edition 2024)
- Replace single-arm match with `if let`
- Never suppress clippy lints without a comment explaining why

### Formatting
- Always run `cargo fmt --all` before committing
- CI enforces `cargo fmt --all -- --check`

### Forbidden patterns in production code (NOT tests)
- No `.unwrap()` — use `?` or `.ok_or()`/`.map_err()`
- No `.expect()` — same as unwrap, use proper error handling
- No `panic!()`, `todo!()`, `unimplemented!()`
- No `#[allow(clippy::*)]` without extreme justification
- No `#[allow(dead_code)]` — remove unused code instead

### Dependency rules
- Don't duplicate crates in both `[dependencies]` and `[dev-dependencies]`
- `sha2` and `hex` are already in `[dependencies]` — use them directly

### Test conventions
- `.unwrap()` is fine in tests
- Don't hardcode version strings — use `env!("CARGO_PKG_VERSION")`
- Doc tests must compile — test them with `cargo test --doc`

### Edition 2024 features available
- `let_chains`: `if cond && let Some(x) = opt { ... }` is valid
- Use this instead of nested `if` + `if let`

### GitHub Actions
- Use `actions/cache@v4`, `actions/upload-artifact@v4` (NOT v3)
- CI runs on stable, beta, and nightly Rust across Linux/macOS/Windows
