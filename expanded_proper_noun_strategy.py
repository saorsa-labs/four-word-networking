#!/usr/bin/env python3
"""
Expanded proper noun replacement strategy with more categories to achieve better coverage.
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

    # Major world cities
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

    # Common first names (international)
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

    # Rivers and water bodies
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

    # Mountains and landmarks
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
            "mackenzie",
            "matterhorn",
            "mckinley",
            "mont",
            "olympus",
            "rockies",
            "sahara",
            "ural",
            "vesuvius",
        ]
    )

    # Historical figures
    proper_nouns.update(
        [
            "alexander",
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

    # Brands and companies
    proper_nouns.update(
        [
            "adobe",
            "amazon",
            "apple",
            "boeing",
            "canon",
            "cisco",
            "coca",
            "dell",
            "disney",
            "fedex",
            "ferrari",
            "ford",
            "google",
            "honda",
            "hyundai",
            "ibm",
            "intel",
            "mercedes",
            "microsoft",
            "nike",
            "nokia",
            "oracle",
            "pepsi",
            "samsung",
            "siemens",
            "sony",
            "toyota",
            "volkswagen",
            "walmart",
            "xerox",
        ]
    )

    # Mythological and cultural
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

    # Planets, moons, and space
    proper_nouns.update(
        [
            "ceres",
            "earth",
            "europa",
            "ganymede",
            "jupiter",
            "mars",
            "mercury",
            "neptune",
            "pluto",
            "saturn",
            "titan",
            "uranus",
            "venus",
        ]
    )

    # Academic subjects and fields
    proper_nouns.update(
        [
            "algebra",
            "anatomy",
            "anthropology",
            "archaeology",
            "astronomy",
            "biology",
            "botany",
            "calculus",
            "chemistry",
            "economics",
            "geology",
            "geometry",
            "linguistics",
            "physics",
            "psychology",
            "sociology",
            "trigonometry",
            "zoology",
        ]
    )

    return proper_nouns


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


def main():
    words = read_wordlist("GOLD_WORDLIST.txt")
    proper_nouns = get_expanded_proper_nouns()
    word_to_min_prefix = calculate_min_prefix_lengths(words)

    # Find problematic words
    problematic_words = [
        word for word, min_len in word_to_min_prefix.items() if min_len > 5
    ]

    print("=" * 80)
    print("EXPANDED PROPER NOUN REPLACEMENT STRATEGY")
    print("=" * 80)
    print()

    print("AVAILABLE RESOURCES:")
    print(f"Current dictionary size: {len(words)} words")
    print(
        f"Words requiring 6+ characters: {len(problematic_words)} words ({len(problematic_words)/len(words)*100:.1f}%)"
    )
    print(f"Total proper noun candidates: {len(proper_nouns)} words")
    print()

    # Test which proper nouns would have unique prefixes
    all_words_extended = set(words) | proper_nouns
    unique_proper_nouns = []

    for noun in proper_nouns:
        noun_prefix = noun[:5]
        # Check conflicts with existing dictionary
        conflicts_existing = [w for w in words if w.startswith(noun_prefix)]
        # Check conflicts with other proper nouns
        conflicts_nouns = [
            n for n in proper_nouns if n != noun and n.startswith(noun_prefix)
        ]

        if not conflicts_existing and not conflicts_nouns:
            unique_proper_nouns.append(noun)

    print("UNIQUE PROPER NOUNS ANALYSIS:")
    print(f"Proper nouns with unique 5-char prefixes: {len(unique_proper_nouns)}")
    print(
        f"Coverage potential: {min(len(unique_proper_nouns), len(problematic_words))} words can be replaced"
    )
    print(
        f"After replacement, problematic words remaining: {max(0, len(problematic_words) - len(unique_proper_nouns))}"
    )
    print()

    if len(unique_proper_nouns) >= len(problematic_words):
        print(
            "✅ SUCCESS: We have enough unique proper nouns to replace ALL problematic words!"
        )
        print(
            f"   We can achieve 100% 5-character uniqueness while maintaining 4096 dictionary size."
        )
        success_rate = 100.0
    else:
        success_rate = (
            (len(words) - (len(problematic_words) - len(unique_proper_nouns)))
            / len(words)
            * 100
        )
        print(
            f"📊 PARTIAL SUCCESS: We can achieve {success_rate:.1f}% 5-character uniqueness"
        )
        print(
            f"   {len(problematic_words) - len(unique_proper_nouns)} words would still need 6+ characters"
        )

    print()

    # Show sample categories of unique proper nouns
    print("SAMPLE UNIQUE PROPER NOUNS BY CATEGORY:")

    categories = {
        "Countries": ["afghanistan", "albania", "algeria", "andorra", "angola"],
        "Cities": ["abuja", "accra", "addis", "adelaide", "algiers"],
        "Names": ["aaron", "abdul", "abraham", "adrian", "ahmed"],
        "Rivers": ["amazon", "brahmaputra", "colorado", "columbia", "congo"],
        "Mountains": ["aconcagua", "alps", "andes", "annapurna", "appalachian"],
        "Historical": ["alexander", "archimedes", "aristotle", "augustus", "bach"],
        "Brands": ["adobe", "amazon", "apple", "boeing", "canon"],
        "Mythology": ["apollo", "ares", "artemis", "athena", "atlas"],
    }

    for category, samples in categories.items():
        available_in_category = [
            noun for noun in samples if noun in unique_proper_nouns
        ]
        print(
            f"{category:12s}: {len([n for n in unique_proper_nouns if any(n.startswith(s[:3]) for s in samples)])} available"
        )

    print()
    print("IMPLEMENTATION RECOMMENDATION:")
    if success_rate >= 95:
        print("✅ PROCEED with proper noun replacement strategy")
        print("   - Maintain exactly 4096 dictionary size")
        print("   - Replace problematic words with unique proper nouns")
        print("   - Achieve near-perfect 5-character autocomplete")
    else:
        print("⚠️  HYBRID APPROACH recommended:")
        print("   - Use proper nouns for major conflicts")
        print("   - Consider some synonym replacements for remaining conflicts")
        print("   - May need to accept some 6-character requirements")


if __name__ == "__main__":
    main()
