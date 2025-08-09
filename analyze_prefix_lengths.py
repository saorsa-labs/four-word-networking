#!/usr/bin/env python3
"""
Analyze prefix lengths needed for unique word identification in GOLD_WORDLIST.txt

This script determines how many characters are needed to uniquely identify each word
in the dictionary, providing statistics on prefix length requirements.
"""

import sys
from collections import defaultdict
from pathlib import Path


def read_wordlist(file_path):
    """Read words from the wordlist file - each line contains one word."""
    words = []
    with open(file_path, "r", encoding="utf-8") as f:
        for line in f:
            word = line.strip()
            if word:  # Skip empty lines
                words.append(word.lower())
    return words


def calculate_min_prefix_lengths(words):
    """Calculate minimum prefix length needed to uniquely identify each word."""
    word_to_min_prefix = {}

    for word in words:
        min_length = 1

        # Try increasing prefix lengths until we find a unique one
        while min_length <= len(word):
            prefix = word[:min_length]

            # Check if this prefix is unique among all words
            conflicts = [w for w in words if w != word and w.startswith(prefix)]

            if not conflicts:
                word_to_min_prefix[word] = min_length
                break

            min_length += 1

        # If we couldn't find a unique prefix, the word needs its full length
        if word not in word_to_min_prefix:
            word_to_min_prefix[word] = len(word)

    return word_to_min_prefix


def analyze_prefix_statistics(word_to_min_prefix):
    """Generate statistics about prefix length requirements."""
    prefix_lengths = list(word_to_min_prefix.values())
    length_distribution = defaultdict(int)

    for length in prefix_lengths:
        length_distribution[length] += 1

    total_words = len(prefix_lengths)

    print("=" * 60)
    print("WORD PREFIX ANALYSIS RESULTS")
    print("=" * 60)
    print(f"Total words analyzed: {total_words}")
    print(f"Average prefix length needed: {sum(prefix_lengths) / total_words:.2f}")
    print(f"Minimum prefix length needed: {min(prefix_lengths)}")
    print(f"Maximum prefix length needed: {max(prefix_lengths)}")
    print()

    print("PREFIX LENGTH DISTRIBUTION:")
    print("Length | Count | Percentage | Cumulative %")
    print("-" * 45)

    cumulative = 0
    for length in sorted(length_distribution.keys()):
        count = length_distribution[length]
        percentage = (count / total_words) * 100
        cumulative += percentage
        print(f"{length:6d} | {count:5d} | {percentage:9.2f}% | {cumulative:11.2f}%")

    print()

    # Find words that need longer prefixes
    long_prefix_words = [
        (word, length) for word, length in word_to_min_prefix.items() if length >= 5
    ]
    long_prefix_words.sort(key=lambda x: x[1], reverse=True)

    if long_prefix_words:
        print("WORDS REQUIRING 5+ CHARACTER PREFIXES:")
        print("Word                    | Prefix Length | Prefix")
        print("-" * 55)
        for word, length in long_prefix_words[:20]:  # Show top 20
            prefix = word[:length]
            print(f"{word:22s} | {length:13d} | {prefix}")

        if len(long_prefix_words) > 20:
            print(f"... and {len(long_prefix_words) - 20} more words")
        print()

    # Show some examples of words with different prefix lengths
    print("EXAMPLES BY PREFIX LENGTH:")
    for length in sorted(set(prefix_lengths))[:10]:  # Show first 10 lengths
        examples = [
            (word, word[:length])
            for word, plen in word_to_min_prefix.items()
            if plen == length
        ]
        if examples:
            word, prefix = examples[0]  # Just show first example
            print(f"Length {length}: '{word}' → '{prefix}'")


def find_prefix_conflicts(words):
    """Find groups of words that share common prefixes."""
    prefix_groups = defaultdict(list)

    # Group words by their 3-character prefixes
    for word in words:
        if len(word) >= 3:
            prefix = word[:3]
            prefix_groups[prefix].append(word)

    # Find prefixes with multiple words
    conflicts = {
        prefix: words_list
        for prefix, words_list in prefix_groups.items()
        if len(words_list) > 1
    }

    if conflicts:
        print("TOP PREFIX CONFLICTS (3-character prefixes with multiple words):")
        print("Prefix | Count | Words (first 5)")
        print("-" * 60)

        # Sort by number of conflicts
        sorted_conflicts = sorted(
            conflicts.items(), key=lambda x: len(x[1]), reverse=True
        )

        for prefix, words_list in sorted_conflicts[:15]:  # Show top 15
            words_preview = ", ".join(words_list[:5])
            if len(words_list) > 5:
                words_preview += f" ... (+{len(words_list) - 5} more)"
            print(f"{prefix:6s} | {len(words_list):5d} | {words_preview}")


def main():
    wordlist_path = Path("GOLD_WORDLIST.txt")

    if not wordlist_path.exists():
        print(f"Error: {wordlist_path} not found!")
        print("Make sure the script is run from the project root directory.")
        sys.exit(1)

    print("Reading wordlist...")
    words = read_wordlist(wordlist_path)
    print(f"Loaded {len(words)} words")

    print("\nCalculating minimum prefix lengths...")
    word_to_min_prefix = calculate_min_prefix_lengths(words)

    print("\nAnalyzing statistics...")
    analyze_prefix_statistics(word_to_min_prefix)

    print("\nFinding prefix conflicts...")
    find_prefix_conflicts(words)

    print("\n" + "=" * 60)
    print("ANALYSIS COMPLETE")
    print("=" * 60)


if __name__ == "__main__":
    main()
