from tree import HuffmanTree
from coder import distinct_sorted_chars, frequencies, total_symbols
from coder import original_bits, fixed_width, compressed_bits
from coder import weighted_path_length, saving_permille


def yes_no(b):
    if b:
        return "yes"
    return "no"


def analyze(label, text):
    print("== " + label + " ==")
    print("textlen " + str(len(text)))

    chars = distinct_sorted_chars(text)
    print("distinct " + str(len(chars)))

    counts = frequencies(text, chars)
    total = total_symbols(counts)
    print("total " + str(total))

    syms = []
    for j in range(len(chars)):
        syms.append(j)

    tree = HuffmanTree()
    tree.build(syms, counts)
    print("nodes " + str(len(tree.sym)))
    print("internal " + str(tree.internal_count()))

    lengths = tree.code_lengths()

    print("CODES")
    for j in range(len(chars)):
        c = chars[j]
        shown = c
        if c == " ":
            shown = "_"
        print(shown + " " + str(counts[j]) + " " + str(lengths[j]))

    width = fixed_width(len(chars))
    orig = original_bits(counts, width)
    comp = compressed_bits(counts, lengths)
    check = weighted_path_length(tree.freq, tree.sym, len(tree.sym))

    print("width " + str(width))
    print("origbits " + str(orig))
    print("huffbits " + str(comp))
    print("pathcheck " + str(check))
    print("match " + yes_no(comp == check))
    print("savepermille " + str(saving_permille(orig, comp)))

    longest = 0
    for j in range(len(chars)):
        if lengths[j] > longest:
            longest = lengths[j]
    print("maxcode " + str(longest))


def main():
    samples = []
    samples.append(("ABRA", "abracadabra"))
    samples.append(("MISS", "mississippi"))
    samples.append(("ROSE", "a rose is a rose is a rose"))
    samples.append(("GEESE", "the geese see the cheese"))
    samples.append(("SOLO", "zzzz"))
    samples.append(("PAIR", "abab"))

    for pair in samples:
        analyze(pair[0], pair[1])


main()
