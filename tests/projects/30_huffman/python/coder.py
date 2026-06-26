# Frequency analysis and the numeric reporting around a Huffman tree.
# Characters are mapped to integer symbol ids in sorted character order, so
# the whole pipeline (ids, tree, code lengths) is deterministic and prints as
# plain integers / explicit strings only.


def char_at(text, i):
    # One-character substring; Iron-friendly way to read a "char" as a str.
    return text[i:i + 1]


def distinct_sorted_chars(text):
    seen = {}
    for i in range(len(text)):
        c = char_at(text, i)
        if c not in seen:
            seen[c] = 1
    chars = []
    for k in sorted(seen.keys()):
        chars.append(k)
    return chars


def frequencies(text, chars):
    # Count of each character, aligned with the chars list by index.
    counts = []
    for j in range(len(chars)):
        counts.append(0)
    index = {}
    for j in range(len(chars)):
        index[chars[j]] = j
    for i in range(len(text)):
        c = char_at(text, i)
        j = index[c]
        counts[j] = counts[j] + 1
    return counts


def total_symbols(counts):
    total = 0
    for c in counts:
        total = total + c
    return total


def original_bits(counts, width):
    # Fixed-width baseline: every symbol costs `width` bits.
    return total_symbols(counts) * width


def fixed_width(n):
    # Minimum bits to give n distinct symbols a flat code (at least 1).
    if n <= 1:
        return 1
    bits = 0
    span = 1
    while span < n:
        span = span * 2
        bits = bits + 1
    return bits


def compressed_bits(counts, lengths):
    # Sum over symbols of frequency * Huffman code length.
    total = 0
    for j in range(len(counts)):
        total = total + counts[j] * lengths[j]
    return total


def weighted_path_length(freq, sym, count):
    # Same quantity computed from raw tree weights: sum of internal node
    # frequencies equals the total number of emitted bits. Used as a cross
    # check against compressed_bits.
    total = 0
    for i in range(count):
        if sym[i] == -1:
            total = total + freq[i]
    return total


def saving_permille(original, compressed):
    # How much smaller, in tenths of a percent, to stay in integer land.
    if original == 0:
        return 0
    return (original - compressed) * 1000 // original
