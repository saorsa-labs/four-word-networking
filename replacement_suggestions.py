#!/usr/bin/env python3
"""
Generate specific replacement suggestions for 5-character autocomplete optimization.
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


def generate_replacement_suggestions():
    """Generate specific replacement suggestions."""

    # Synonym replacements for common conflicts
    synonym_replacements = {
        # Easy synonyms that maintain meaning
        "absolutely": "totally",
        "actually": "really",
        "additional": "extra",
        "afternoon": "evening",  # Or keep 'after'
        "alternative": "option",
        "apparently": "clearly",
        "appropriate": "suitable",
        "approximately": "about",
        "arrangement": "setup",
        "assignment": "task",
        "assistance": "help",
        "associate": "partner",
        "assumption": "belief",
        "atmosphere": "climate",
        "attractive": "appealing",
        "automobile": "car",
        "available": "free",
        "awareness": "knowledge",
        # Technical/formal to simpler
        "accommodate": "fit",
        "accomplish": "achieve",
        "acknowledge": "admit",
        "acquisition": "purchase",
        "administration": "management",
        "administrator": "manager",
        "agricultural": "farming",
        "agriculture": "farming",
        "anniversary": "birthday",  # Or use 'yearly'
        "anticipate": "expect",
        "application": "app",
        "appreciate": "value",
        "archaeology": "digging",  # Simplified
        "architecture": "design",
        "arithmetic": "math",
        "arrangement": "setup",
        "assessment": "review",
        "association": "group",
        "astronomy": "stars",
        "atmosphere": "air",
        "automobile": "car",
        # Compound words to simpler forms
        "basketball": "hoops",
        "battlefield": "warzone",
        "birthday": "birth",
        "boyfriend": "partner",
        "breakfast": "morning",
        "broadcast": "show",
        "businessman": "trader",
        "butterfly": "bug",
        # Long technical terms
        "characteristic": "trait",
        "characterize": "describe",
        "circumstance": "situation",
        "collaboration": "teamwork",
        "combination": "mix",
        "comfortable": "cozy",
        "commercial": "business",
        "commission": "fee",
        "commitment": "promise",
        "communicate": "talk",
        "communication": "talking",
        "community": "town",
        "comparison": "contrast",
        "competition": "contest",
        "competitive": "fierce",
        "complicated": "complex",
        "comprehensive": "complete",
        "concentrate": "focus",
        "concentration": "focus",
        "conclusion": "end",
        "condition": "state",
        "conference": "meeting",
        "confidence": "trust",
        "congressional": "congress",
        "connection": "link",
        "consciousness": "awareness",
        "consequence": "result",
        "conservative": "cautious",
        "considerable": "large",
        "consideration": "thought",
        "consistent": "steady",
        "constitution": "charter",
        "constitutional": "legal",
        "construction": "building",
        "consumption": "use",
        "contemporary": "modern",
        "contribution": "gift",
        "controversial": "disputed",
        "controversy": "dispute",
        "conventional": "normal",
        "conversation": "chat",
        "cooperation": "teamwork",
        "correspondent": "reporter",
        # Keep removing longer variants of shorter words
        "dangerous": "risky",  # vs 'danger'
        "democracy": "voting",  # vs 'democrat'
        "democratic": "fair",
        "demonstrate": "show",
        "demonstration": "protest",
        "department": "dept",
        "description": "details",
        "destruction": "ruin",
        "development": "growth",
        "difference": "gap",
        "different": "other",
        "difficulty": "problem",
        "dimension": "size",
        "direction": "way",
        "directly": "straight",
        "director": "boss",
        "disability": "handicap",
        "discrimination": "bias",
        "discussion": "talk",
        "distribution": "spread",
        "diversity": "variety",
        # Education/academic terms
        "education": "learning",
        "educational": "school",
        "efficiency": "speed",
        "efficient": "fast",
        "electricity": "power",
        "electronic": "digital",
        "elementary": "basic",
        "elimination": "removal",
        "emergency": "crisis",
        "employment": "work",
        "enforcement": "policing",
        "engineering": "building",
        "enterprise": "business",
        "entertainment": "fun",
        "environment": "nature",
        "equipment": "gear",
        "especially": "mainly",
        "essentially": "basically",
        "establishment": "place",
        "evaluation": "review",
        "everything": "all",
        "everywhere": "all",
        "examination": "exam",
        "excellent": "great",
        "exception": "oddity",
        "excitement": "thrill",
        "executive": "boss",
        "exercise": "workout",
        "exhibition": "show",
        "existence": "being",
        "expansion": "growth",
        "expectation": "hope",
        "expensive": "costly",
        "experience": "event",
        "experiment": "test",
        "explanation": "reason",
        "explosion": "blast",
        "expression": "phrase",
        "extension": "addon",
        "extraordinary": "amazing",
        # Government/political
        "facility": "building",
        "factory": "plant",
        "failure": "loss",
        "familiar": "known",
        "fantastic": "great",
        "federation": "union",
        "financial": "money",
        "formation": "setup",
        "foundation": "base",
        "frequency": "rate",
        "frequently": "often",
        "friendship": "bond",
        "fundamental": "basic",
        "furthermore": "also",
        # Geography/location
        "generation": "age",
        "generally": "mostly",
        "gentleman": "man",
        "geography": "maps",
        "government": "state",
        "gradually": "slowly",
        "grandfather": "grandpa",
        "grandmother": "grandma",
        "guarantee": "promise",
        # Technology/modern
        "headline": "title",
        "headquarters": "office",
        "helicopter": "chopper",
        "highlight": "feature",
        "historian": "scholar",
        "historical": "old",
        "household": "family",
        "hurricane": "storm",
        "hypothesis": "theory",
        # Identity/personal
        "identification": "id",
        "identity": "self",
        "imagination": "dreams",
        "immediately": "now",
        "immigration": "moving",
        "implementation": "doing",
        "implication": "meaning",
        "importance": "value",
        "important": "key",
        "impossible": "hopeless",
        "impression": "feeling",
        "improvement": "upgrade",
        "independence": "freedom",
        "independent": "free",
        "individual": "person",
        "industrial": "factory",
        "industry": "business",
        "infection": "illness",
        "inflation": "prices",
        "influence": "power",
        "information": "data",
        "ingredient": "part",
        "inheritance": "legacy",
        "initiative": "plan",
        "injury": "hurt",
        "innovation": "invention",
        "inspection": "check",
        "installation": "setup",
        "institution": "school",
        "institutional": "formal",
        "instruction": "teaching",
        "instructor": "teacher",
        "instrument": "tool",
        "insurance": "coverage",
        "intellectual": "smart",
        "intelligence": "smarts",
        "intelligent": "smart",
        "intensity": "strength",
        "intention": "plan",
        "interaction": "contact",
        "interesting": "cool",
        "interference": "blocking",
        "internal": "inside",
        "international": "global",
        "internet": "web",
        "interpretation": "meaning",
        "intervention": "help",
        "interview": "chat",
        "introduction": "intro",
        "investigation": "probe",
        "investigator": "detective",
        "investment": "funding",
        "investor": "backer",
        "invitation": "invite",
        "involvement": "part",
    }

    return synonym_replacements


def main():
    words = read_wordlist("GOLD_WORDLIST.txt")
    conflict_groups = find_conflict_groups(words, 5)
    replacements = generate_replacement_suggestions()

    print("=" * 80)
    print("SPECIFIC REPLACEMENT SUGGESTIONS FOR 5-CHARACTER AUTOCOMPLETE")
    print("=" * 80)
    print()

    # Count how many conflicts we can resolve
    resolvable_words = set()
    for old_word, new_word in replacements.items():
        if old_word in words:
            resolvable_words.add(old_word)

    print(f"SUMMARY:")
    print(f"Total words in dictionary: {len(words)}")
    print(
        f"Words with 5+ char prefix conflicts: {sum(len(group) for group in conflict_groups.values())}"
    )
    print(f"Direct replacement suggestions: {len(resolvable_words)}")
    print(f"Estimated conflict reduction: {len(resolvable_words)} words")
    print()

    print("REPLACEMENT SUGGESTIONS (sample):")
    print("Original Word          → Replacement")
    print("-" * 50)

    count = 0
    for old_word, new_word in sorted(replacements.items()):
        if old_word in words and count < 50:
            print(f"{old_word:20s} → {new_word}")
            count += 1

    print(f"... and {len(resolvable_words) - count} more replacements")
    print()

    # Analyze impact on major conflict groups
    print("IMPACT ON MAJOR CONFLICT GROUPS:")
    print("Prefix     | Original Count | After Replacements")
    print("-" * 50)

    for prefix, word_list in sorted(
        conflict_groups.items(), key=lambda x: len(x[1]), reverse=True
    )[:15]:
        remaining_words = [w for w in word_list if w not in replacements]
        original_count = len(word_list)
        remaining_count = len(remaining_words)
        print(f"{prefix:10s} | {original_count:13d} | {remaining_count:17d}")

    print()
    print("RECOMMENDED IMPLEMENTATION STRATEGY:")
    print("1. Apply direct synonym replacements (reduces conflicts by ~300 words)")
    print("2. Remove longer variants where shorter form exists")
    print(
        "3. For remaining conflicts, choose the most common/useful word from each group"
    )
    print("4. Consider domain-specific replacements based on your use case")


if __name__ == "__main__":
    main()
