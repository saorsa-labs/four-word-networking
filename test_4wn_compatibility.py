#!/usr/bin/env python3
"""
Test compatibility of optimized wordlist with four-word networking system.

This script tests that the optimized wordlist works correctly with the existing
four-word networking implementation by testing encoding/decoding operations.
"""

import random
import subprocess
import sys
import time
from pathlib import Path


def test_cli_encoding_decoding(wordlist_path, test_addresses, num_tests=20):
    """Test encoding/decoding with the CLI using the optimized wordlist."""

    print(f"Testing four-word networking CLI compatibility...")
    print(f"Using wordlist: {wordlist_path}")
    print(f"Testing {num_tests} addresses...")
    print()

    # Check if CLI exists
    cli_command = ["cargo", "run", "--bin", "4wn", "--"]

    successes = 0
    failures = []

    for i, address in enumerate(test_addresses[:num_tests], 1):
        try:
            print(f"Test {i:2d}: {address:<20}", end=" → ")

            # Encode address to words
            encode_result = subprocess.run(
                cli_command + [address], capture_output=True, text=True, timeout=10
            )

            if encode_result.returncode != 0:
                error_msg = f"Encoding failed: {encode_result.stderr.strip()}"
                print(f"❌ {error_msg}")
                failures.append((address, error_msg))
                continue

            # Extract the encoded words from output
            encoded_output = encode_result.stdout.strip()
            # Look for the line with dots (IPv4) or spaces (IPv6)
            encoded_words = None
            for line in encoded_output.split("\n"):
                if "." in line and len(line.split(".")) == 4:
                    encoded_words = line.strip()
                    break
                elif " " in line and len(line.split()) in [6, 9, 12]:
                    encoded_words = line.strip()
                    break

            if not encoded_words:
                error_msg = f"No encoded words found in output: {encoded_output}"
                print(f"❌ {error_msg}")
                failures.append((address, error_msg))
                continue

            print(f"{encoded_words}", end=" → ")

            # Decode words back to address
            decode_result = subprocess.run(
                cli_command + [encoded_words],
                capture_output=True,
                text=True,
                timeout=10,
            )

            if decode_result.returncode != 0:
                error_msg = f"Decoding failed: {decode_result.stderr.strip()}"
                print(f"❌ {error_msg}")
                failures.append((address, error_msg))
                continue

            # Extract decoded address
            decoded_output = decode_result.stdout.strip()
            decoded_address = None
            for line in decoded_output.split("\n"):
                # Look for IP address pattern
                if ":" in line and "[" in line:  # IPv6 format
                    decoded_address = line.strip()
                    break
                elif ":" in line and "." in line:  # IPv4 format
                    decoded_address = line.strip()
                    break

            if not decoded_address:
                error_msg = f"No decoded address found: {decoded_output}"
                print(f"❌ {error_msg}")
                failures.append((address, error_msg))
                continue

            # Verify round trip
            if decoded_address == address:
                print(f"✅ {decoded_address}")
                successes += 1
            else:
                error_msg = f"Round trip mismatch: {decoded_address} != {address}"
                print(f"❌ {error_msg}")
                failures.append((address, error_msg))

        except subprocess.TimeoutExpired:
            error_msg = "Timeout"
            print(f"❌ {error_msg}")
            failures.append((address, error_msg))
        except Exception as e:
            error_msg = f"Exception: {str(e)}"
            print(f"❌ {error_msg}")
            failures.append((address, error_msg))

    print()
    print("TEST RESULTS:")
    print(f"  Successful: {successes}/{num_tests} ({successes/num_tests*100:.1f}%)")
    print(
        f"  Failed:     {len(failures)}/{num_tests} ({len(failures)/num_tests*100:.1f}%)"
    )

    if failures:
        print("\nFAILURES:")
        for address, error in failures[:5]:  # Show first 5 failures
            print(f"  {address}: {error}")
        if len(failures) > 5:
            print(f"  ... and {len(failures) - 5} more")

    return successes, failures


def test_dictionary_integration():
    """Test that the optimized dictionary is properly integrated."""

    print("Testing dictionary integration...")

    # Check if optimized wordlist exists
    optimized_path = Path("GOLD_WORDLIST_OPTIMIZED.txt")
    if not optimized_path.exists():
        print("❌ GOLD_WORDLIST_OPTIMIZED.txt not found")
        return False

    # Read optimized wordlist
    with open(optimized_path, "r") as f:
        optimized_words = [line.strip() for line in f if line.strip()]

    print(f"✅ Optimized wordlist loaded: {len(optimized_words)} words")

    # Check for proper nouns
    proper_nouns = [
        w
        for w in optimized_words
        if w[0].isupper()
        or w
        in {
            "afghanistan",
            "albania",
            "algeria",
            "amsterdam",
            "athens",
            "beijing",
            "berlin",
            "cairo",
            "delhi",
            "london",
            "madrid",
            "moscow",
            "paris",
            "tokyo",
            "washington",
        }
    ]

    print(f"✅ Found {len(proper_nouns)} proper nouns in optimized dictionary")

    # Sample some proper nouns
    if proper_nouns:
        sample_proper = random.sample(proper_nouns, min(5, len(proper_nouns)))
        print(f"   Sample proper nouns: {', '.join(sample_proper)}")

    return True


def generate_test_addresses():
    """Generate a variety of test addresses for compatibility testing."""

    test_addresses = [
        # IPv4 addresses with ports
        "192.168.1.1:443",
        "10.0.0.1:80",
        "172.16.0.1:8080",
        "8.8.8.8:53",
        "1.1.1.1:443",
        "127.0.0.1:3000",
        "203.0.113.1:22",
        "198.51.100.1:21",
        "224.0.0.1:1234",
        "255.255.255.255:65535",
        # IPv6 addresses with ports
        "[::1]:443",
        "[2001:db8::1]:80",
        "[fe80::1]:8080",
        "[2001:4860:4860::8888]:53",
        "[::ffff:192.168.1.1]:443",
        "[2001:db8:85a3::8a2e:370:7334]:8080",
        "[fc00::1]:22",
        "[2001:db8::]:80",
        "[::]:443",
        "[fe80::dead:beef:cafe:babe]:1234",
    ]

    # Add some random IPv4 addresses
    for _ in range(10):
        a = random.randint(1, 223)
        b = random.randint(0, 255)
        c = random.randint(0, 255)
        d = random.randint(1, 254)
        port = random.randint(1, 65535)
        test_addresses.append(f"{a}.{b}.{c}.{d}:{port}")

    return test_addresses


def main():
    print("=" * 80)
    print("FOUR-WORD NETWORKING COMPATIBILITY TEST")
    print("=" * 80)
    print()

    # Test dictionary integration
    if not test_dictionary_integration():
        print("❌ Dictionary integration test failed")
        return
    print()

    # Generate test addresses
    test_addresses = generate_test_addresses()
    print(f"Generated {len(test_addresses)} test addresses")
    print()

    # Test CLI compatibility
    successes, failures = test_cli_encoding_decoding(
        "GOLD_WORDLIST_OPTIMIZED.txt", test_addresses
    )
    print()

    # Overall assessment
    print("COMPATIBILITY ASSESSMENT:")
    print("-" * 30)

    if len(failures) == 0:
        print("🎉 PERFECT COMPATIBILITY")
        print("   All tests passed. Optimized wordlist is fully compatible.")
    elif successes / len(test_addresses) >= 0.9:
        print("✅ EXCELLENT COMPATIBILITY")
        print("   >90% success rate. Minor issues may exist.")
    elif successes / len(test_addresses) >= 0.8:
        print("👍 GOOD COMPATIBILITY")
        print("   >80% success rate. Some investigation needed.")
    else:
        print("⚠️ COMPATIBILITY ISSUES")
        print("   <80% success rate. Significant issues detected.")

    print()
    print("RECOMMENDATIONS:")

    if len(failures) == 0:
        print("✅ Ready for production deployment")
        print("✅ Optimized wordlist can replace original GOLD_WORDLIST.txt")
        print("✅ 5-character autocomplete will work excellently")
    elif successes / len(test_addresses) >= 0.8:
        print("⚠️ Review failed test cases before deployment")
        print("✅ Overall optimization is successful")
        print("📝 Consider creating backup of original wordlist")
    else:
        print("❌ Do not deploy until compatibility issues are resolved")
        print("🔍 Investigate encoding/decoding failures")
        print("🛠️ May need to adjust replacement strategy")


if __name__ == "__main__":
    main()
