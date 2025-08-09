from collections import defaultdict

path = "GOLD_WORDLIST_OPTIMIZED.txt"  # point to your file
with open(path) as f:
    words = [ln.strip() for ln in f if ln.strip()]


def p5(w):
    return w[:5] if len(w) >= 5 else w  # my rule


groups = defaultdict(list)
for i, w in enumerate(words):
    groups[p5(w)].append((i, w))

collisions = {k: v for k, v in groups.items() if len(v) > 1}
print("Total words:", len(words))
print("5-char collision groups:", len(collisions))
for k, v in list(collisions.items())[:25]:
    print(k, "->", [w for _, w in v])
