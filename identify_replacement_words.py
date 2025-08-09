#!/usr/bin/env python3
"""
Identify words that need replacement for 5-character autocomplete system.

This script finds words that require more than 5 characters for unique identification
and suggests replacement strategies.
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


def find_conflict_groups(words, max_prefix_length=5):
    """Find groups of words that conflict within the given prefix length."""
    prefix_to_words = defaultdict(list)

    for word in words:
        prefix = word[:max_prefix_length]
        prefix_to_words[prefix].append(word)

    # Return only groups with conflicts (more than one word)
    conflict_groups = {
        prefix: word_list
        for prefix, word_list in prefix_to_words.items()
        if len(word_list) > 1
    }

    return conflict_groups


def analyze_replacement_candidates(words, word_to_min_prefix, max_prefix_length=5):
    """Analyze which words need replacement for the given prefix length."""

    # Words that need more than max_prefix_length characters
    problematic_words = [
        (word, min_len)
        for word, min_len in word_to_min_prefix.items()
        if min_len > max_prefix_length
    ]

    print("=" * 80)
    print(
        f"WORDS REQUIRING MORE THAN {max_prefix_length} CHARACTERS FOR UNIQUE IDENTIFICATION"
    )
    print("=" * 80)
    print(
        f"Total problematic words: {len(problematic_words)} out of {len(words)} ({len(problematic_words)/len(words)*100:.1f}%)"
    )
    print()

    # Group by required length
    by_length = defaultdict(list)
    for word, min_len in problematic_words:
        by_length[min_len].append(word)

    print("BREAKDOWN BY REQUIRED LENGTH:")
    for length in sorted(by_length.keys()):
        count = len(by_length[length])
        print(f"Length {length:2d}: {count:4d} words ({count/len(words)*100:.1f}%)")
    print()

    # Find conflict groups
    conflict_groups = find_conflict_groups(words, max_prefix_length)

    print("TOP CONFLICT GROUPS (words sharing same 5-character prefix):")
    print("Prefix     | Count | Words")
    print("-" * 60)

    # Sort by number of conflicts
    sorted_conflicts = sorted(
        conflict_groups.items(), key=lambda x: len(x[1]), reverse=True
    )

    for prefix, word_list in sorted_conflicts[:20]:  # Show top 20
        words_str = ", ".join(word_list[:8])  # First 8 words
        if len(word_list) > 8:
            words_str += f" ... (+{len(word_list) - 8} more)"
        print(f"{prefix:10s} | {len(word_list):5d} | {words_str}")

    print()

    # Categorize replacement strategies
    print("REPLACEMENT STRATEGY ANALYSIS:")
    print("-" * 40)

    # Strategy 1: Remove longer variants when shorter exists
    removal_candidates = []
    keep_shorter_variants = []

    for prefix, word_list in conflict_groups.items():
        if len(word_list) == 2:
            shorter, longer = sorted(word_list, key=len)
            if longer.startswith(shorter):
                removal_candidates.append((longer, shorter, "longer variant"))
                keep_shorter_variants.append(shorter)

    print(f"1. REMOVE LONGER VARIANTS ({len(removal_candidates)} candidates):")
    print("   Remove Word          | Keep Word     | Reason")
    print("   " + "-" * 50)
    for remove, keep, reason in removal_candidates[:15]:
        print(f"   {remove:18s} | {keep:12s} | {reason}")
    if len(removal_candidates) > 15:
        print(f"   ... and {len(removal_candidates) - 15} more")
    print()

    # Strategy 2: Look for semantic groups that could be consolidated
    semantic_groups = defaultdict(list)
    common_roots = [
        "admin",
        "repre",
        "const",
        "commu",
        "conce",
        "chara",
        "agric",
        "insti",
    ]

    for root in common_roots:
        matching_words = [word for word in words if word.startswith(root)]
        if len(matching_words) > 1:
            semantic_groups[root] = matching_words

    print("2. SEMANTIC GROUPS FOR POTENTIAL CONSOLIDATION:")
    for root, word_list in semantic_groups.items():
        if len(word_list) > 2:
            print(f"   {root}: {', '.join(word_list[:6])}")
            if len(word_list) > 6:
                print(f"        ... and {len(word_list) - 6} more")
    print()

    # Strategy 3: Find completely unrelated words that conflict
    unrelated_conflicts = []
    for prefix, word_list in conflict_groups.items():
        if len(word_list) <= 4:  # Focus on smaller conflicts
            # Check if words are semantically unrelated (simple heuristic)
            unrelated = True
            for i, word1 in enumerate(word_list):
                for word2 in word_list[i + 1 :]:
                    # If one word is contained in another, they're related
                    if word1 in word2 or word2 in word1:
                        unrelated = False
                        break
                if not unrelated:
                    break

            if unrelated:
                unrelated_conflicts.append((prefix, word_list))

    print("3. UNRELATED WORD CONFLICTS (may need synonym replacement):")
    for prefix, word_list in unrelated_conflicts[:10]:
        print(f"   {prefix}: {', '.join(word_list)}")
    print()

    # Generate summary statistics
    total_removals_needed = len(problematic_words)
    easy_removals = len(removal_candidates)
    difficult_cases = total_removals_needed - easy_removals

    print("SUMMARY:")
    print(f"Total words needing replacement: {total_removals_needed}")
    print(f"Easy removals (longer variants): {easy_removals}")
    print(f"Difficult cases (need synonyms): {difficult_cases}")
    print(f"Dictionary reduction needed: {total_removals_needed/len(words)*100:.1f}%")

    return {
        "problematic_words": problematic_words,
        "conflict_groups": conflict_groups,
        "removal_candidates": removal_candidates,
        "semantic_groups": dict(semantic_groups),
        "unrelated_conflicts": unrelated_conflicts,
    }


def main():
    wordlist_path = Path("GOLD_WORDLIST.txt")

    if not wordlist_path.exists():
        print(f"Error: {wordlist_path} not found!")
        sys.exit(1)

    print("Reading wordlist...")
    words = read_wordlist(wordlist_path)
    print(f"Loaded {len(words)} words")

    print("\nCalculating minimum prefix lengths...")
    word_to_min_prefix = calculate_min_prefix_lengths(words)

    print("\nAnalyzing replacement candidates...")
    analysis = analyze_replacement_candidates(words, word_to_min_prefix, 5)

    print("\n" + "=" * 80)
    print("ANALYSIS COMPLETE")
    print("=" * 80)


if __name__ == "__main__":
    main()
