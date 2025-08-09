#!/usr/bin/env python3
"""
Generate optimized wordlist using proper noun replacements for 5-character autocomplete.

This script creates an optimized version of GOLD_WORDLIST.txt by replacing problematic
words with proper nouns that have unique 5-character prefixes.
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


def get_expanded_proper_nouns():
    """Comprehensive list of proper nouns across many categories."""

    proper_nouns = set()

    # Countries and territories
    proper_nouns.update(
        [
            "afghanistan",
            "albania",
            "algeria",
            "andorra",
            "angola",
            "argentina",
            "armenia",
            "australia",
            "austria",
            "azerbaijan",
            "bahamas",
            "bahrain",
            "bangladesh",
            "barbados",
            "belarus",
            "belgium",
            "belize",
            "benin",
            "bhutan",
            "bolivia",
            "bosnia",
            "botswana",
            "brazil",
            "brunei",
            "bulgaria",
            "burkina",
            "burundi",
            "cambodia",
            "cameroon",
            "canada",
            "cape",
            "central",
            "chad",
            "chile",
            "china",
            "colombia",
            "comoros",
            "congo",
            "costa",
            "croatia",
            "cuba",
            "cyprus",
            "czechia",
            "denmark",
            "djibouti",
            "dominica",
            "dominican",
            "ecuador",
            "egypt",
            "salvador",
            "equatorial",
            "eritrea",
            "estonia",
            "eswatini",
            "ethiopia",
            "fiji",
            "finland",
            "france",
            "gabon",
            "gambia",
            "georgia",
            "ghana",
            "greece",
            "grenada",
            "guatemala",
            "guinea",
            "guyana",
            "haiti",
            "honduras",
            "hungary",
            "iceland",
            "india",
            "indonesia",
            "iran",
            "iraq",
            "ireland",
            "israel",
            "italy",
            "ivory",
            "jamaica",
            "japan",
            "jordan",
            "kazakhstan",
            "kenya",
            "kiribati",
            "korea",
            "kosovo",
            "kuwait",
            "kyrgyzstan",
            "laos",
            "latvia",
            "lebanon",
            "lesotho",
            "liberia",
            "libya",
            "liechtenstein",
            "lithuania",
            "luxembourg",
            "madagascar",
            "malawi",
            "malaysia",
            "maldives",
            "mali",
            "malta",
            "marshall",
            "mauritania",
            "mauritius",
            "mexico",
            "micronesia",
            "moldova",
            "monaco",
            "mongolia",
            "montenegro",
            "morocco",
            "mozambique",
            "myanmar",
            "namibia",
            "nauru",
            "nepal",
            "netherlands",
            "zealand",
            "nicaragua",
            "niger",
            "nigeria",
            "north",
            "norway",
            "oman",
            "pakistan",
            "palau",
            "palestine",
            "panama",
            "papua",
            "paraguay",
            "peru",
            "philippines",
            "poland",
            "portugal",
            "qatar",
            "romania",
            "russia",
            "rwanda",
            "samoa",
            "marino",
            "sao",
            "saudi",
            "senegal",
            "serbia",
            "seychelles",
            "sierra",
            "singapore",
            "slovakia",
            "slovenia",
            "solomon",
            "somalia",
            "south",
            "spain",
            "lanka",
            "sudan",
            "suriname",
            "sweden",
            "switzerland",
            "syria",
            "taiwan",
            "tajikistan",
            "tanzania",
            "thailand",
            "togo",
            "tonga",
            "trinidad",
            "tunisia",
            "turkey",
            "turkmenistan",
            "tuvalu",
            "uganda",
            "ukraine",
            "emirates",
            "united",
            "uruguay",
            "uzbekistan",
            "vanuatu",
            "vatican",
            "venezuela",
            "vietnam",
            "yemen",
            "zambia",
            "zimbabwe",
        ]
    )

    # Major cities
    proper_nouns.update(
        [
            "abuja",
            "accra",
            "addis",
            "adelaide",
            "algiers",
            "almaty",
            "amman",
            "amsterdam",
            "ankara",
            "antananarivo",
            "apia",
            "ashgabat",
            "asmara",
            "astana",
            "asuncion",
            "athens",
            "atlanta",
            "auckland",
            "baghdad",
            "baku",
            "bamako",
            "bandar",
            "bangalore",
            "bangkok",
            "bangui",
            "banjul",
            "barcelona",
            "basseterre",
            "beijing",
            "beirut",
            "belgrade",
            "belmopan",
            "berlin",
            "bern",
            "bishkek",
            "bissau",
            "bogota",
            "brasilia",
            "bratislava",
            "brazzaville",
            "bridgetown",
            "brisbane",
            "brussels",
            "bucharest",
            "budapest",
            "buenos",
            "bujumbura",
            "cairo",
            "calgary",
            "canberra",
            "cape",
            "caracas",
            "castries",
            "cayenne",
            "chicago",
            "chisinau",
            "cologne",
            "colombo",
            "conakry",
            "copenhagen",
            "dakar",
            "dallas",
            "damascus",
            "delhi",
            "dhaka",
            "dili",
            "djibouti",
            "dodoma",
            "doha",
            "dublin",
            "dushanbe",
            "edinburgh",
            "florence",
            "frankfurt",
            "freetown",
            "funafuti",
            "gaborone",
            "geneva",
            "georgetown",
            "glasgow",
            "guatemala",
            "hamburg",
            "hanoi",
            "harare",
            "havana",
            "helsinki",
            "honiara",
            "houston",
            "islamabad",
            "istanbul",
            "jakarta",
            "juba",
            "kabul",
            "kampala",
            "kathmandu",
            "khartoum",
            "kigali",
            "kingston",
            "kingstown",
            "kinshasa",
            "kuala",
            "kuwait",
            "lagos",
            "leipzig",
            "libreville",
            "lilongwe",
            "lima",
            "lisbon",
            "ljubljana",
            "lome",
            "london",
            "luanda",
            "lusaka",
            "madrid",
            "majuro",
            "malabo",
            "male",
            "managua",
            "manama",
            "manila",
            "maputo",
            "marseille",
            "maseru",
            "mbabane",
            "melbourne",
            "mexico",
            "miami",
            "milan",
            "minsk",
            "mogadishu",
            "monaco",
            "monrovia",
            "montevideo",
            "montreal",
            "moroni",
            "moscow",
            "mumbai",
            "munich",
            "muscat",
            "nairobi",
            "nassau",
            "ndjamena",
            "niamey",
            "nicosia",
            "nouakchott",
            "nukualofa",
            "oslo",
            "ottawa",
            "ouagadougou",
            "panama",
            "paramaribo",
            "paris",
            "perth",
            "phnom",
            "podgorica",
            "port",
            "porto",
            "prague",
            "praia",
            "pretoria",
            "pristina",
            "pyongyang",
            "quito",
            "rabat",
            "reykjavik",
            "riga",
            "riyadh",
            "rome",
            "roseau",
            "saint",
            "san",
            "sanaa",
            "santiago",
            "santo",
            "sao",
            "sarajevo",
            "seattle",
            "seoul",
            "shanghai",
            "singapore",
            "skopje",
            "sofia",
            "stockholm",
            "sucre",
            "suva",
            "sydney",
            "taipei",
            "tallinn",
            "tarawa",
            "tashkent",
            "tbilisi",
            "tegucigalpa",
            "tehran",
            "thimphu",
            "tirana",
            "tokyo",
            "toronto",
            "tripoli",
            "tunis",
            "ulaanbaatar",
            "vaduz",
            "valletta",
            "vancouver",
            "vatican",
            "venice",
            "victoria",
            "vienna",
            "vientiane",
            "vilnius",
            "warsaw",
            "washington",
            "wellington",
            "windhoek",
            "yaounde",
            "yaren",
            "yerevan",
            "zagreb",
            "zurich",
        ]
    )

    # Common names
    proper_nouns.update(
        [
            "aaron",
            "abdul",
            "abraham",
            "adam",
            "adrian",
            "ahmed",
            "akira",
            "alan",
            "albert",
            "alejandro",
            "alexander",
            "alfred",
            "alice",
            "amanda",
            "amy",
            "andrew",
            "angela",
            "anna",
            "anthony",
            "antonio",
            "arthur",
            "barbara",
            "beatrice",
            "benjamin",
            "bernard",
            "betty",
            "brian",
            "bruce",
            "carlos",
            "carol",
            "catherine",
            "charles",
            "chris",
            "christian",
            "christine",
            "christopher",
            "claudia",
            "daniel",
            "david",
            "deborah",
            "dennis",
            "diana",
            "diane",
            "dmitri",
            "donald",
            "donna",
            "dorothy",
            "douglas",
            "edward",
            "elena",
            "elizabeth",
            "emily",
            "eric",
            "eugene",
            "evelyn",
            "fernando",
            "francesco",
            "francis",
            "francisco",
            "frank",
            "gary",
            "george",
            "giovanni",
            "giuseppe",
            "gloria",
            "gonzalez",
            "gregory",
            "harold",
            "helen",
            "helena",
            "henry",
            "ibrahim",
            "irene",
            "isaac",
            "isabella",
            "jackie",
            "jacob",
            "jacques",
            "james",
            "janet",
            "jason",
            "jean",
            "jeffrey",
            "jennifer",
            "jeremy",
            "jerry",
            "jessica",
            "joan",
            "johannes",
            "jonathan",
            "jorge",
            "jose",
            "joseph",
            "joshua",
            "juan",
            "judith",
            "julia",
            "julie",
            "justin",
            "karen",
            "kathleen",
            "kenneth",
            "kevin",
            "kimberly",
            "larry",
            "laura",
            "lawrence",
            "leonardo",
            "linda",
            "lisa",
            "lorenzo",
            "louis",
            "luis",
            "mahmoud",
            "margaret",
            "maria",
            "marie",
            "mario",
            "mark",
            "martin",
            "martinez",
            "mary",
            "matthew",
            "maxime",
            "melissa",
            "michael",
            "michelle",
            "miguel",
            "mohammed",
            "nancy",
            "natasha",
            "nicholas",
            "nicolas",
            "nicole",
            "olivier",
            "patricia",
            "patrick",
            "paul",
            "peter",
            "philippe",
            "rachel",
            "rafael",
            "raymond",
            "rebecca",
            "ricardo",
            "richard",
            "robert",
            "roberto",
            "rodrigo",
            "ronald",
            "ruth",
            "sandra",
            "sarah",
            "scott",
            "sebastian",
            "sharon",
            "stephanie",
            "stephen",
            "steven",
            "susan",
            "theodore",
            "thomas",
            "timothy",
            "valentina",
            "valerie",
            "victoria",
            "vincent",
            "vladimir",
            "walter",
            "william",
            "wolfgang",
            "xavier",
        ]
    )

    # Rivers and landmarks
    proper_nouns.update(
        [
            "amazon",
            "brahmaputra",
            "colorado",
            "columbia",
            "congo",
            "danube",
            "dnieper",
            "euphrates",
            "ganges",
            "hudson",
            "indus",
            "irrawaddy",
            "jordan",
            "lena",
            "mackenzie",
            "mekong",
            "mississippi",
            "missouri",
            "murray",
            "niger",
            "nile",
            "orinoco",
            "rhine",
            "rio",
            "thames",
            "tigris",
            "volga",
            "yangtze",
            "yellow",
            "yukon",
            "zambezi",
        ]
    )

    # Mountains
    proper_nouns.update(
        [
            "aconcagua",
            "alps",
            "andes",
            "annapurna",
            "appalachian",
            "atlas",
            "caucasus",
            "denali",
            "everest",
            "fuji",
            "himalaya",
            "kilimanjaro",
            "matterhorn",
            "mckinley",
            "mont",
            "olympus",
            "rockies",
            "ural",
            "vesuvius",
        ]
    )

    # Historical figures
    proper_nouns.update(
        [
            "archimedes",
            "aristotle",
            "augustus",
            "bach",
            "beethoven",
            "caesar",
            "churchill",
            "cleopatra",
            "columbus",
            "confucius",
            "copernicus",
            "darwin",
            "edison",
            "einstein",
            "franklin",
            "galileo",
            "gandhi",
            "hamilton",
            "homer",
            "jefferson",
            "leonardo",
            "lincoln",
            "michelangelo",
            "mozart",
            "napoleon",
            "newton",
            "picasso",
            "plato",
            "shakespeare",
            "socrates",
            "tesla",
            "voltaire",
            "washington",
        ]
    )

    # Brands
    proper_nouns.update(
        [
            "adobe",
            "boeing",
            "canon",
            "cisco",
            "coca",
            "dell",
            "disney",
            "fedex",
            "ferrari",
            "ford",
            "hyundai",
            "ibm",
            "mercedes",
            "nike",
            "oracle",
            "pepsi",
            "siemens",
            "volkswagen",
            "walmart",
            "xerox",
        ]
    )

    # Mythology
    proper_nouns.update(
        [
            "apollo",
            "ares",
            "artemis",
            "athena",
            "atlas",
            "buddha",
            "ceres",
            "diana",
            "hades",
            "hera",
            "hercules",
            "hermes",
            "isis",
            "juno",
            "jupiter",
            "mars",
            "mercury",
            "minerva",
            "neptune",
            "odin",
            "osiris",
            "pluto",
            "poseidon",
            "saturn",
            "thor",
            "venus",
            "zeus",
        ]
    )

    return proper_nouns


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


def select_optimal_replacements(words, proper_nouns, target_prefix_length=5):
    """Select optimal word replacements using proper nouns."""

    # Calculate current problematic words
    word_to_min_prefix = calculate_min_prefix_lengths(words)
    problematic_words = [
        word
        for word, min_len in word_to_min_prefix.items()
        if min_len > target_prefix_length
    ]

    # Find unique proper nouns (no conflicts with existing words or other proper nouns)
    unique_proper_nouns = []
    for noun in proper_nouns:
        noun_prefix = noun[:target_prefix_length]
        # Check conflicts with existing dictionary
        conflicts_existing = [w for w in words if w.startswith(noun_prefix)]
        # Check conflicts with other proper nouns
        conflicts_nouns = [
            n for n in proper_nouns if n != noun and n.startswith(noun_prefix)
        ]

        if not conflicts_existing and not conflicts_nouns:
            unique_proper_nouns.append(noun)

    print(
        f"Found {len(unique_proper_nouns)} unique proper nouns for {len(problematic_words)} problematic words"
    )

    # Get conflict groups to prioritize high-impact replacements
    conflict_groups = find_conflict_groups(words, target_prefix_length)

    # Strategy: Replace words that resolve the most conflicts first
    replacements = {}
    used_proper_nouns = set()
    remaining_problematic = set(problematic_words)

    # Sort conflict groups by size (largest first)
    sorted_conflicts = sorted(
        conflict_groups.items(), key=lambda x: len(x[1]), reverse=True
    )

    for prefix, conflict_words in sorted_conflicts:
        # Find problematic words in this conflict group
        group_problematic = [w for w in conflict_words if w in remaining_problematic]

        if group_problematic and unique_proper_nouns:
            # Choose the most problematic word to replace (longest or most complex)
            word_to_replace = max(
                group_problematic, key=lambda w: (word_to_min_prefix[w], len(w))
            )

            # Find a suitable proper noun replacement
            available_nouns = [
                n for n in unique_proper_nouns if n not in used_proper_nouns
            ]
            if available_nouns:
                replacement_noun = available_nouns[0]
                replacements[word_to_replace] = replacement_noun
                used_proper_nouns.add(replacement_noun)
                remaining_problematic.discard(word_to_replace)
                print(
                    f"Replace '{word_to_replace}' → '{replacement_noun}' (resolves {len(conflict_words)} conflicts in '{prefix}' group)"
                )

    # Fill remaining slots with any problematic words we can replace
    remaining_nouns = [n for n in unique_proper_nouns if n not in used_proper_nouns]
    for word in list(remaining_problematic):
        if remaining_nouns:
            replacement = remaining_nouns.pop(0)
            replacements[word] = replacement
            used_proper_nouns.add(replacement)
            remaining_problematic.discard(word)

    print(f"\nTotal replacements planned: {len(replacements)}")
    print(f"Problematic words remaining: {len(remaining_problematic)}")

    return replacements


def generate_optimized_wordlist(input_file, output_file, replacements):
    """Generate the optimized wordlist with replacements."""

    words = read_wordlist(input_file)

    # Apply replacements
    optimized_words = []
    replacement_count = 0

    for word in words:
        if word in replacements:
            optimized_words.append(replacements[word])
            replacement_count += 1
        else:
            optimized_words.append(word)

    # Verify we still have 4096 words
    assert len(optimized_words) == len(
        words
    ), f"Word count mismatch: {len(optimized_words)} vs {len(words)}"

    # Sort the optimized words for consistency
    optimized_words.sort()

    # Write to output file
    with open(output_file, "w", encoding="utf-8") as f:
        for word in optimized_words:
            f.write(f"{word}\n")

    print(f"\nOptimized wordlist generated:")
    print(f"  Input: {input_file} ({len(words)} words)")
    print(f"  Output: {output_file} ({len(optimized_words)} words)")
    print(f"  Replacements applied: {replacement_count}")

    return optimized_words


def main():
    input_file = "GOLD_WORDLIST.txt"
    output_file = "GOLD_WORDLIST_OPTIMIZED.txt"

    if not Path(input_file).exists():
        print(f"Error: {input_file} not found!")
        sys.exit(1)

    print("=" * 80)
    print("GENERATING OPTIMIZED WORDLIST FOR 5-CHARACTER AUTOCOMPLETE")
    print("=" * 80)
    print()

    # Load data
    print("Loading wordlist and proper nouns...")
    words = read_wordlist(input_file)
    proper_nouns = get_expanded_proper_nouns()

    print(f"Current dictionary: {len(words)} words")
    print(f"Proper noun candidates: {len(proper_nouns)} words")
    print()

    # Find optimal replacements
    print("Calculating optimal replacements...")
    replacements = select_optimal_replacements(words, proper_nouns, 5)
    print()

    # Generate optimized wordlist
    print("Generating optimized wordlist...")
    optimized_words = generate_optimized_wordlist(input_file, output_file, replacements)

    print(f"\n✅ Successfully generated {output_file}")
    print("\nNext steps:")
    print("1. Run validation script to verify 5-character uniqueness")
    print("2. Test compatibility with four-word networking system")
    print("3. Generate analysis report")


if __name__ == "__main__":
    main()
