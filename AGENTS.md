# four-word-networking

Rust library (`four-word-networking`, lib `four_word_networking`) and `4wn` CLI
that turn socket addresses into words and back. Used by x0x (pins `2.7.0` from
crates.io).

- **IPv4 + port** → exactly 4 words: the 48 bits (32 IP + 16 port) map to four
  12-bit indices into a 4,096-word dictionary. Perfect round-trip.
- **IPv6 + port** → 6, 9 or 12 words (groups of 4 words / 48 bits),
  choosing the shortest that fits after category-based compression
  (`ipv6_compression.rs`, `ipv6_pattern_feistel.rs`, `ipv6_perfect_patterns.rs`).
- **Identities** (`identity_encoder.rs`): the first 48 bits of an x0x AgentId /
  UserId hash as 4 words; 8 words (`agent @ user`) means a human-backed agent.
- CLI output is lowercase words separated by spaces; decoding also accepts dots.

## Compatibility constraint
The dictionary is `GOLD_WORDLIST_OPTIMIZED.txt` (repo root, 4,096 lines),
compiled in via `include_str!` in `src/dictionary4k.rs`. Word order is the wire
format: reordering, replacing or deduplicating words, or changing the bit
packing, silently changes every published address and identity. Treat such a
change as a breaking release with an ADR.

## Layout
- Public API in `src/lib.rs`: `FourWordAdaptiveEncoder` (main entry, IPv4 or
  IPv6), `FourWordEncoder` (IPv4), `FourWordIpv6Encoder`, `IdentityEncoder`,
  `FourWordError`.
- `src/bin/4wn.rs` is the CLI (`cargo run --bin 4wn -- 192.168.1.1:443`,
  `cargo run --bin 4wn -- "[::1]:443"`, or pass words to decode). Other
  `src/bin/debug_*` and `4wn_*` binaries are ad-hoc tools.
- `4wn-cli/` is a separate interactive CLI crate (not a workspace member).
- Many top-level `*.py`, `*.rs`, `*.md` reports and scripts are historical
  experiments from dictionary generation; they are not part of the build.

## Build and test
No justfile. CI (`.github/workflows/comprehensive_tests.yml`) runs on Linux
stable + nightly, macOS and Windows:
- `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo doc --all-features --no-deps`
- `cargo nextest run --lib --all-features`, `cargo nextest run --tests --all-features`,
  `cargo test --doc --all-features`, property tests (`src/property_tests.rs`,
  `tests/property_tests.rs`), `cargo bench --all-features`, `cargo audit`,
  and an aarch64 `cargo zigbuild`.
- `./run_main_tests.sh` runs the core suite locally.

## Conventions
- Edition 2024: use let-chains (`if cond && let Some(x) = opt`) rather than
  nested `if` / `if let`.
- No `#[allow(dead_code)]`; delete unused code. Justify any `#[allow(clippy::…)]`
  in a comment.
- `sha2` and `hex` are already regular dependencies; don't duplicate crates in
  `[dev-dependencies]`.
- Tests use `env!("CARGO_PKG_VERSION")`, not hard-coded version strings.

## Architecture decisions
Before changing encoding, the dictionary, public APIs or compatibility
behaviour, check `docs/adr/`. New or changed decisions go in a Proposed ADR
(`docs/adr/TEMPLATE.md`); Accepted ADRs are immutable (CI `adr-governance`
enforces this) and only humans mark an ADR Accepted.
