#!/bin/bash

# Test improved interactive mode

echo "Testing improved interactive mode..."

# Test 1: Type an IP address
echo -e "Testing IP input: 192.168.1.1\n"
echo -e "192.168.1.1\nquit" | timeout 2s cargo run --bin 4wn_improved 2>/dev/null | grep -A2 "192.168.1.1"

# Test 2: Type single-letter word "a"
echo -e "\nTesting single-letter word 'a':"
echo -e "a\nquit" | timeout 2s cargo run --bin 4wn_improved 2>/dev/null | grep -A2 "Word Mode"

# Test 3: Type partial words
echo -e "\nTesting partial word completion:"
echo -e "corr\tquit" | timeout 2s cargo run --bin 4wn_improved 2>/dev/null | grep -A2 "Hints"

echo -e "\nDone testing!"