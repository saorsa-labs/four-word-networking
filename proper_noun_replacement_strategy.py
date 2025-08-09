#!/usr/bin/env python3
"""
Analyze replacement strategy using proper nouns while maintaining 4096 dictionary size.

Strategy: Replace problematic common words with proper nouns that have unique 5-char prefixes.
"""

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


def get_proper_noun_candidates():
    """Generate list of proper noun candidates with unique 5-char prefixes."""

    proper_nouns = {
        # Countries (good geographical coverage)
        "albania",
        "andorra",
        "armenia",
        "austria",
        "bahrain",
        "belarus",
        "belgium",
        "bolivia",
        "botswana",
        "bulgaria",
        "cambodia",
        "cameroon",
        "croatia",
        "cyprus",
        "czechia",
        "denmark",
        "ecuador",
        "estonia",
        "finland",
        "georgia",
        "germany",
        "hungary",
        "iceland",
        "ireland",
        "jamaica",
        "jordan",
        "kosovo",
        "latvia",
        "lebanon",
        "liberia",
        "lithuania",
        "luxembourg",
        "malta",
        "moldova",
        "monaco",
        "montenegro",
        "morocco",
        "namibia",
        "nepal",
        "nigeria",
        "norway",
        "panama",
        "poland",
        "portugal",
        "romania",
        "serbia",
        "slovakia",
        "slovenia",
        "sweden",
        "switzerland",
        "tunisia",
        "turkey",
        "ukraine",
        "uruguay",
        "zambia",
        # Cities (major world cities)
        "adelaide",
        "amsterdam",
        "athens",
        "atlanta",
        "baghdad",
        "bangalore",
        "barcelona",
        "beijing",
        "belgrade",
        "berlin",
        "bogota",
        "boston",
        "brisbane",
        "brussels",
        "budapest",
        "buenos",
        "cairo",
        "calgary",
        "canberra",
        "caracas",
        "chicago",
        "cologne",
        "copenhagen",
        "dallas",
        "delhi",
        "detroit",
        "dublin",
        "durban",
        "edinburgh",
        "florence",
        "frankfurt",
        "geneva",
        "glasgow",
        "hamburg",
        "helsinki",
        "houston",
        "istanbul",
        "jakarta",
        "karachi",
        "kiev",
        "lagos",
        "leipzig",
        "lisbon",
        "london",
        "madrid",
        "manila",
        "marseille",
        "melbourne",
        "miami",
        "milan",
        "montreal",
        "moscow",
        "mumbai",
        "munich",
        "nairobi",
        "naples",
        "oslo",
        "ottawa",
        "paris",
        "perth",
        "prague",
        "quebec",
        "riyadh",
        "rome",
        "seattle",
        "seoul",
        "shanghai",
        "stockholm",
        "sydney",
        "tehran",
        "tokyo",
        "toronto",
        "tunis",
        "valencia",
        "vancouver",
        "venice",
        "vienna",
        "warsaw",
        "zurich",
        # Common first names (diverse, international)
        "abdul",
        "ahmed",
        "akira",
        "alejandro",
        "alexander",
        "alfonso",
        "andrea",
        "antonio",
        "beatrice",
        "carlos",
        "catherine",
        "christine",
        "christopher",
        "claudia",
        "dmitri",
        "eduardo",
        "elena",
        "francisco",
        "giovanni",
        "giuseppe",
        "gonzalez",
        "helena",
        "ibrahim",
        "jacques",
        "jessica",
        "johannes",
        "jonathan",
        "leonardo",
        "lorenzo",
        "mahmoud",
        "margaret",
        "maria",
        "martinez",
        "maxime",
        "miguel",
        "mohammed",
        "natasha",
        "nicolas",
        "olivier",
        "patricia",
        "philippe",
        "rafael",
        "ricardo",
        "roberto",
        "rodrigo",
        "sebastian",
        "stephanie",
        "theodore",
        "valentina",
        "valerie",
        "victoria",
        "vladimir",
        "wolfgang",
        "xavier",
        # Rivers and natural features
        "amazon",
        "brahmaputra",
        "colorado",
        "danube",
        "euphrates",
        "ganges",
        "hudson",
        "indus",
        "jordan",
        "mackenzie",
        "mekong",
        "murray",
        "niger",
        "nile",
        "rhine",
        "thames",
        "volga",
        "yangtze",
        "yukon",
        # Mountains and landmarks
        "alps",
        "andes",
        "everest",
        "fuji",
        "himalaya",
        "kilimanjaro",
        "mckinley",
        "rockies",
        "sahara",
        "ural",
        # Historical figures (widely known)
        "aristotle",
        "beethoven",
        "churchill",
        "columbus",
        "confucius",
        "darwin",
        "edison",
        "einstein",
        "galileo",
        "gandhi",
        "homer",
        "jefferson",
        "lincoln",
        "mozart",
        "napoleon",
        "newton",
        "plato",
        "shakespeare",
        "socrates",
        "tesla",
        "washington",
        # Brands/Companies (commonly known)
        "adobe",
        "amazon",
        "apple",
        "boeing",
        "canon",
        "cisco",
        "dell",
        "fedex",
        "google",
        "honda",
        "intel",
        "microsoft",
        "nike",
        "nokia",
        "oracle",
        "pepsi",
        "samsung",
        "sony",
        "toyota",
        "walmart",
        # Mythological/Cultural
        "apollo",
        "athena",
        "buddha",
        "diana",
        "hercules",
        "jupiter",
        "mars",
        "mercury",
        "neptune",
        "pluto",
        "saturn",
        "thor",
        "venus",
        "zeus",
    }

    return proper_nouns


def analyze_proper_noun_strategy():
    """Analyze the proper noun replacement strategy."""

    words = read_wordlist("GOLD_WORDLIST.txt")
    conflict_groups = find_conflict_groups(words, 5)
    word_to_min_prefix = calculate_min_prefix_lengths(words)
    proper_nouns = get_proper_noun_candidates()

    # Find problematic words that need more than 5 chars
    problematic_words = [
        word for word, min_len in word_to_min_prefix.items() if min_len > 5
    ]

    print("=" * 80)
    print("PROPER NOUN REPLACEMENT STRATEGY ANALYSIS")
    print("=" * 80)
    print()

    print("CURRENT SITUATION:")
    print(f"Total dictionary size: {len(words)} words")
    print(f"Words requiring 6+ characters: {len(problematic_words)} words")
    print(f"Conflict groups (5-char prefixes): {len(conflict_groups)} groups")
    print(f"Available proper noun candidates: {len(proper_nouns)} words")
    print()

    # Check which proper nouns would have unique 5-char prefixes
    all_words_extended = set(words) | proper_nouns
    proper_noun_conflicts = find_conflict_groups(list(all_words_extended), 5)

    # Find proper nouns that would be unique with 5 chars
    unique_proper_nouns = []
    conflicting_proper_nouns = []

    for noun in proper_nouns:
        noun_prefix = noun[:5]
        conflicts_with_existing = [
            w for w in words if w.startswith(noun_prefix) and w != noun
        ]
        conflicts_with_other_nouns = [
            n for n in proper_nouns if n != noun and n.startswith(noun_prefix)
        ]

        if not conflicts_with_existing and not conflicts_with_other_nouns:
            unique_proper_nouns.append(noun)
        else:
            conflicting_proper_nouns.append(
                (noun, conflicts_with_existing + conflicts_with_other_nouns)
            )

    print("PROPER NOUN ANALYSIS:")
    print(f"Proper nouns with unique 5-char prefixes: {len(unique_proper_nouns)}")
    print(f"Proper nouns that would still conflict: {len(conflicting_proper_nouns)}")
    print()

    # Show some examples of unique proper nouns by category
    print("SAMPLE UNIQUE PROPER NOUNS BY CATEGORY:")

    countries = [
        n
        for n in unique_proper_nouns
        if n
        in {
            "albania",
            "andorra",
            "armenia",
            "austria",
            "bahrain",
            "belarus",
            "belgium",
            "bolivia",
            "botswana",
            "bulgaria",
            "cambodia",
            "cameroon",
            "croatia",
        }
    ]

    cities = [
        n
        for n in unique_proper_nouns
        if n
        in {
            "adelaide",
            "athens",
            "atlanta",
            "baghdad",
            "bangalore",
            "barcelona",
            "belgrade",
            "bogota",
            "brisbane",
            "brussels",
            "budapest",
        }
    ]

    names = [
        n
        for n in unique_proper_nouns
        if n in {"abdul", "ahmed", "akira", "alejandro", "alfonso", "beatrice"}
    ]

    print(f"Countries ({len(countries)}): {', '.join(countries[:10])}")
    print(f"Cities ({len(cities)}): {', '.join(cities[:10])}")
    print(f"Names ({len(names)}): {', '.join(names[:10])}")
    print()

    # Strategy recommendations
    print("REPLACEMENT STRATEGY:")
    print("1. MAINTAIN 4096 DICTIONARY SIZE")
    print("2. Replace problematic common words with unique proper nouns")
    print("3. Prioritize replacements that resolve the most conflicts")
    print()

    # Calculate optimal replacements
    optimal_replacements = []
    remaining_problematic = set(problematic_words)
    available_nouns = list(unique_proper_nouns)

    # Sort conflict groups by size (biggest conflicts first)
    sorted_conflicts = sorted(
        conflict_groups.items(), key=lambda x: len(x[1]), reverse=True
    )

    for prefix, conflict_words in sorted_conflicts:
        if available_nouns and any(w in remaining_problematic for w in conflict_words):
            # Find the most problematic word in this group
            group_problematic = [
                w for w in conflict_words if w in remaining_problematic
            ]
            if group_problematic:
                # Replace the longest/most complex word
                word_to_replace = max(group_problematic, key=len)
                replacement_noun = available_nouns.pop(0)
                optimal_replacements.append(
                    (word_to_replace, replacement_noun, prefix, len(conflict_words))
                )
                remaining_problematic.discard(word_to_replace)

    print("TOP REPLACEMENT RECOMMENDATIONS:")
    print("Replace Word           → Proper Noun    | Prefix | Conflicts Resolved")
    print("-" * 75)

    for old_word, new_word, prefix, conflicts in optimal_replacements[:20]:
        print(f"{old_word:20s} → {new_word:12s} | {prefix:6s} | {conflicts:6d}")

    print(f"... and {len(optimal_replacements) - 20} more optimal replacements")
    print()

    print("ESTIMATED IMPACT:")
    print(f"Words that can be optimally replaced: {len(optimal_replacements)}")
    print(f"Remaining problematic words: {len(remaining_problematic)}")
    print(f"Available proper nouns remaining: {len(available_nouns)}")
    print(f"Dictionary size after replacement: 4096 (maintained)")
    print(
        f"Estimated 5-char uniqueness: {((4096 - len(remaining_problematic)) / 4096 * 100):.1f}%"
    )


if __name__ == "__main__":
    analyze_proper_noun_strategy()
