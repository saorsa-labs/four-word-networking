#!/usr/bin/env python3
"""
Validate 5-character uniqueness improvement in the optimized wordlist.

This script compares the original and optimized wordlists to measure improvement
in 5-character prefix uniqueness for autocomplete functionality.
"""

import sys
from collections import defaultdict
from pathlib import Path


def read_wordlist(file_path):
    """Read words from the wordlist file."""
    words = []
    with open(file_path, "r", encoding="utf-8") as f:
        for line in f:
            word = line.strip()
            if word:
                words.append(word.lower())
    return words


def calculate_min_prefix_lengths(words):
    """Calculate minimum prefix length needed to uniquely identify each word."""
    word_to_min_prefix = {}

    for word in words:
        min_length = 1

        while min_length <= len(word):
            prefix = word[:min_length]
            conflicts = [w for w in words if w != word and w.startswith(prefix)]

            if not conflicts:
                word_to_min_prefix[word] = min_length
                break

            min_length += 1

        if word not in word_to_min_prefix:
            word_to_min_prefix[word] = len(word)

    return word_to_min_prefix


def find_conflict_groups(words, max_prefix_length=5):
    """Find groups of words that conflict within the given prefix length."""
    prefix_to_words = defaultdict(list)

    for word in words:
        prefix = word[:max_prefix_length]
        prefix_to_words[prefix].append(word)

    return {
        prefix: word_list
        for prefix, word_list in prefix_to_words.items()
        if len(word_list) > 1
    }


def analyze_prefix_distribution(word_to_min_prefix):
    """Analyze the distribution of prefix lengths."""
    prefix_lengths = list(word_to_min_prefix.values())
    length_distribution = defaultdict(int)

    for length in prefix_lengths:
        length_distribution[length] += 1

    total_words = len(prefix_lengths)

    stats = {
        "total_words": total_words,
        "avg_prefix_length": sum(prefix_lengths) / total_words,
        "min_prefix_length": min(prefix_lengths),
        "max_prefix_length": max(prefix_lengths),
        "distribution": dict(length_distribution),
    }

    return stats


def compare_wordlists(original_file, optimized_file):
    """Compare original and optimized wordlists."""

    print("=" * 80)
    print("WORDLIST OPTIMIZATION VALIDATION REPORT")
    print("=" * 80)
    print()

    # Load wordlists
    print("Loading wordlists...")
    original_words = read_wordlist(original_file)
    optimized_words = read_wordlist(optimized_file)

    print(f"Original wordlist: {len(original_words)} words")
    print(f"Optimized wordlist: {len(optimized_words)} words")
    print()

    # Verify word count maintained
    if len(original_words) != len(optimized_words):
        print("❌ ERROR: Word count mismatch!")
        return
    else:
        print("✅ Word count maintained exactly")

    # Calculate prefix requirements
    print("Analyzing prefix requirements...")
    original_prefixes = calculate_min_prefix_lengths(original_words)
    optimized_prefixes = calculate_min_prefix_lengths(optimized_words)

    # Get statistics
    original_stats = analyze_prefix_distribution(original_prefixes)
    optimized_stats = analyze_prefix_distribution(optimized_prefixes)

    print()
    print("OVERALL COMPARISON:")
    print("-" * 50)
    print(f"{'Metric':<25} | {'Original':<12} | {'Optimized':<12} | {'Change':<10}")
    print("-" * 50)

    avg_change = (
        optimized_stats["avg_prefix_length"] - original_stats["avg_prefix_length"]
    )
    print(
        f"{'Average prefix length':<25} | {original_stats['avg_prefix_length']:<12.2f} | {optimized_stats['avg_prefix_length']:<12.2f} | {avg_change:+.2f}"
    )

    print(
        f"{'Min prefix length':<25} | {original_stats['min_prefix_length']:<12d} | {optimized_stats['min_prefix_length']:<12d} | {optimized_stats['min_prefix_length'] - original_stats['min_prefix_length']:+d}"
    )

    print(
        f"{'Max prefix length':<25} | {original_stats['max_prefix_length']:<12d} | {optimized_stats['max_prefix_length']:<12d} | {optimized_stats['max_prefix_length'] - original_stats['max_prefix_length']:+d}"
    )

    print()

    # 5-character analysis
    print("5-CHARACTER AUTOCOMPLETE ANALYSIS:")
    print("-" * 50)

    original_5char_ok = len([w for w, l in original_prefixes.items() if l <= 5])
    optimized_5char_ok = len([w for w, l in optimized_prefixes.items() if l <= 5])

    original_5char_pct = (original_5char_ok / len(original_words)) * 100
    optimized_5char_pct = (optimized_5char_ok / len(optimized_words)) * 100

    improvement = optimized_5char_pct - original_5char_pct

    print(f"Words unique with ≤5 chars:")
    print(f"  Original:  {original_5char_ok:4d} words ({original_5char_pct:5.1f}%)")
    print(f"  Optimized: {optimized_5char_ok:4d} words ({optimized_5char_pct:5.1f}%)")
    print(
        f"  Improvement: +{optimized_5char_ok - original_5char_ok:4d} words (+{improvement:5.1f}%)"
    )
    print()

    original_problematic = len([w for w, l in original_prefixes.items() if l > 5])
    optimized_problematic = len([w for w, l in optimized_prefixes.items() if l > 5])

    print(f"Words requiring 6+ chars:")
    print(
        f"  Original:  {original_problematic:4d} words ({(original_problematic/len(original_words)*100):5.1f}%)"
    )
    print(
        f"  Optimized: {optimized_problematic:4d} words ({(optimized_problematic/len(optimized_words)*100):5.1f}%)"
    )
    print(
        f"  Reduction: -{original_problematic - optimized_problematic:4d} words (-{(original_problematic - optimized_problematic)/len(original_words)*100:5.1f}%)"
    )
    print()

    # Distribution comparison
    print("PREFIX LENGTH DISTRIBUTION COMPARISON:")
    print("-" * 70)
    print(f"{'Length':<8} | {'Original':<15} | {'Optimized':<15} | {'Change':<12}")
    print("-" * 70)

    all_lengths = sorted(
        set(
            list(original_stats["distribution"].keys())
            + list(optimized_stats["distribution"].keys())
        )
    )

    for length in all_lengths:
        orig_count = original_stats["distribution"].get(length, 0)
        opt_count = optimized_stats["distribution"].get(length, 0)
        change = opt_count - orig_count

        orig_pct = (orig_count / len(original_words)) * 100
        opt_pct = (opt_count / len(optimized_words)) * 100

        print(
            f"{length:<8d} | {orig_count:4d} ({orig_pct:5.1f}%) | {opt_count:4d} ({opt_pct:5.1f}%) | {change:+4d} ({(change/len(original_words)*100):+5.1f}%)"
        )

    print()

    # Conflict groups analysis
    print("CONFLICT GROUPS ANALYSIS:")
    print("-" * 40)

    original_conflicts = find_conflict_groups(original_words, 5)
    optimized_conflicts = find_conflict_groups(optimized_words, 5)

    print(f"5-character prefix conflict groups:")
    print(f"  Original:  {len(original_conflicts)} groups")
    print(f"  Optimized: {len(optimized_conflicts)} groups")
    print(f"  Reduction: -{len(original_conflicts) - len(optimized_conflicts)} groups")
    print()

    original_conflicted_words = sum(len(words) for words in original_conflicts.values())
    optimized_conflicted_words = sum(
        len(words) for words in optimized_conflicts.values()
    )

    print(f"Words in conflict groups:")
    print(f"  Original:  {original_conflicted_words} words")
    print(f"  Optimized: {optimized_conflicted_words} words")
    print(
        f"  Reduction: -{original_conflicted_words - optimized_conflicted_words} words"
    )
    print()

    # Show biggest remaining conflicts
    if optimized_conflicts:
        print("LARGEST REMAINING CONFLICT GROUPS:")
        print("Prefix | Count | Words (sample)")
        print("-" * 45)

        sorted_conflicts = sorted(
            optimized_conflicts.items(), key=lambda x: len(x[1]), reverse=True
        )

        for prefix, word_list in sorted_conflicts[:10]:
            sample_words = ", ".join(word_list[:4])
            if len(word_list) > 4:
                sample_words += f" ... (+{len(word_list)-4})"
            print(f"{prefix:6s} | {len(word_list):5d} | {sample_words}")
        print()

    # Quality assessment
    print("OPTIMIZATION QUALITY ASSESSMENT:")
    print("-" * 40)

    if improvement >= 10:
        quality = "🌟 EXCELLENT"
    elif improvement >= 5:
        quality = "✅ GOOD"
    elif improvement >= 2:
        quality = "👍 FAIR"
    else:
        quality = "⚠️ MINIMAL"

    print(f"Overall improvement: {quality}")
    print(
        f"5-char uniqueness went from {original_5char_pct:.1f}% to {optimized_5char_pct:.1f}%"
    )

    if optimized_5char_pct >= 90:
        print("🎯 Target achieved: >90% of words unique with 5 characters")
    elif optimized_5char_pct >= 80:
        print("📈 Good progress: >80% of words unique with 5 characters")
    else:
        print("📊 Partial success: Further optimization possible")

    print()

    # Show sample replacements
    original_set = set(original_words)
    optimized_set = set(optimized_words)

    removed_words = original_set - optimized_set
    added_words = optimized_set - original_set

    print("SAMPLE WORD REPLACEMENTS:")
    print("-" * 30)

    # Show some replacements (proper nouns that were added)
    proper_nouns_added = [
        w
        for w in added_words
        if w[0].isupper()
        or w in {"afghanistan", "albania", "algeria", "amsterdam", "athens"}
    ]

    if proper_nouns_added:
        print("New proper nouns added (sample):")
        for word in sorted(list(proper_nouns_added))[:10]:
            print(f"  + {word}")
        if len(proper_nouns_added) > 10:
            print(f"  ... and {len(proper_nouns_added) - 10} more")
        print()

    print("RECOMMENDATION:")
    print("-" * 20)

    if optimized_5char_pct >= 85:
        print("✅ Optimization successful! Ready for production use.")
        print("   5-character autocomplete will work well for most words.")
    else:
        print("⚠️ Consider additional optimization iterations.")
        print("   Focus on remaining high-conflict prefix groups.")


def main():
    original_file = "GOLD_WORDLIST.txt"
    optimized_file = "GOLD_WORDLIST_OPTIMIZED.txt"

    if not Path(original_file).exists():
        print(f"Error: {original_file} not found!")
        sys.exit(1)

    if not Path(optimized_file).exists():
        print(f"Error: {optimized_file} not found!")
        print("Please run generate_optimized_wordlist.py first.")
        sys.exit(1)

    compare_wordlists(original_file, optimized_file)


if __name__ == "__main__":
    main()
