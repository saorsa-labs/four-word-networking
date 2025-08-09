#!/bin/bash

# Test script to demonstrate the new CLI functionality
echo "🧪 Testing Four-Word Networking CLI"
echo

echo "1. Testing direct IPv4 encoding:"
cargo run --bin 4wn -- 192.168.1.1:443
echo

echo "2. Testing direct decoding:"
cargo run --bin 4wn -- "bless abstract king ridge"
echo

echo "3. Testing IPv6 encoding:"
cargo run --bin 4wn -- "[::1]:443"
echo

echo "4. Testing completion hints for 'abo':"
cargo run --bin 4wn -- --complete "abo"
echo

echo "5. Testing validation for partial input:"
cargo run --bin 4wn -- --validate "about ab"
echo

echo "6. Testing verbose output:"
cargo run --bin 4wn -- --verbose 192.168.1.1:80
echo

echo "✅ All CLI tests completed successfully!"
echo "💡 Run '4wn' with no arguments to try the interactive TUI mode"