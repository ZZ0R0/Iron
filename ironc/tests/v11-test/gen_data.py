#!/usr/bin/env python3
# [V11-test] Generateur DETERMINISTE des datasets (seede, sans horodatage/alea non seme).
# Tailles reglees pour ~1-2 s cote CPython sur les programmes qui les scannent en boucle-langage.
# Lance une fois : `python3 gen_data.py`. Les fichiers vont dans ./data/ (gitignore).
import os
import random

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
os.makedirs(DATA, exist_ok=True)

def path(name):
    return os.path.join(DATA, name)

WORDS = ["the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "iron", "rust",
         "python", "code", "loop", "data", "fast", "slow", "prime", "number", "word", "line",
         "byte", "hash", "sum", "sort", "find", "grep", "cut", "map", "key", "val", "node", "tree"]

# --- text_big : ~200k lignes, 3-9 mots ---
random.seed(42)
with open(path("text_big.txt"), "w") as f:
    for _ in range(1000000):
        k = random.randint(3, 9)
        f.write(" ".join(random.choice(WORDS) for _ in range(k)) + "\n")

# --- text_sorted : lignes (mots courts) triees, avec doublons adjacents (pour uniq) ---
random.seed(7)
lines = [random.choice(WORDS) for _ in range(1000000)]
lines.sort()
with open(path("text_sorted.txt"), "w") as f:
    f.write("\n".join(lines) + "\n")

# --- bin_big : ~3 Mo d'octets ASCII imprimables varies (od/cksum/histogram) ---
random.seed(99)
with open(path("bin_big.txt"), "w") as f:
    chunk = "".join(chr(random.randint(32, 126)) for _ in range(60000))
    for _ in range(180):
        f.write(chunk)

# --- csv_big : ~200k lignes "a,b,c,d" numeriques (cut/csvsum) ---
random.seed(11)
with open(path("csv_big.txt"), "w") as f:
    for _ in range(1000000):
        f.write("%d,%d,%d,%d\n" % (random.randint(0, 999), random.randint(0, 999),
                                   random.randint(0, 999), random.randint(0, 999)))

# --- json_big : "[n, n, ...]" ~400k nombres (jsonsum) ---
random.seed(13)
with open(path("json_big.txt"), "w") as f:
    f.write("[")
    f.write(", ".join(str(random.randint(0, 1000)) for _ in range(400000)))
    f.write("]")

# --- sorted_a / sorted_b : listes triees de nombres (comm) ---
random.seed(21)
a = sorted(set(random.randint(0, 500000) for _ in range(150000)))
b = sorted(set(random.randint(0, 500000) for _ in range(150000)))
with open(path("sorted_a.txt"), "w") as f:
    f.write("\n".join(str(x) for x in a) + "\n")
with open(path("sorted_b.txt"), "w") as f:
    f.write("\n".join(str(x) for x in b) + "\n")

# --- words_big : un mot par ligne (anagram) ---
random.seed(33)
with open(path("words_big.txt"), "w") as f:
    for _ in range(150000):
        k = random.randint(3, 7)
        f.write("".join(random.choice("abcdefgh") for _ in range(k)) + "\n")

# --- text_tabs : lignes avec tabulations (expand) ---
random.seed(44)
with open(path("text_tabs.txt"), "w") as f:
    for _ in range(120000):
        k = random.randint(2, 5)
        f.write("\t".join(random.choice(WORDS) for _ in range(k)) + "\n")

# --- nums_med : nombres (naivesort, O(n^2) -> taille moderee) ---
random.seed(55)
with open(path("nums_med.txt"), "w") as f:
    f.write("\n".join(str(random.randint(0, 1000000)) for _ in range(8000)) + "\n")

print("datasets generes dans", DATA)
for n in sorted(os.listdir(DATA)):
    print("  %-18s %8.1f Ko" % (n, os.path.getsize(path(n)) / 1024))
