from tokenizer import tokenize, sentence_count, total_chars, longest, shortest
from stats import frequencies, unique_count, top_n, avg_word_len_x100
from stats import count_over_length, length_histogram, hapax_count
from stats import max_frequency, words_with_frequency


def build_text():
    lines = []
    lines.append("The quick brown fox jumps over the lazy dog.")
    lines.append("The dog was not amused, but the fox was quick!")
    lines.append("Quick thinking saved the brown fox from the lazy dog.")
    lines.append("Was the fox really quick? The dog thought so.")
    return join_lines(lines)


def join_lines(lines):
    out = ""
    for line in lines:
        out = out + line + " "
    return out


def banner(title):
    print("== " + title + " ==")


def main():
    text = build_text()
    words = tokenize(text)

    banner("BASIC")
    print("tokens " + str(len(words)))
    counts = frequencies(words)
    print("unique " + str(unique_count(counts)))
    print("sentences " + str(sentence_count(text)))
    print("totalchars " + str(total_chars(words)))
    print("avglen100 " + str(avg_word_len_x100(words)))
    print("longest " + longest(words) + " " + str(len(longest(words))))
    print("shortest " + shortest(words) + " " + str(len(shortest(words))))

    banner("FREQ")
    for k in sorted(counts.keys()):
        print(k + " " + str(counts[k]))

    banner("TOP5")
    rank = 1
    for pair in top_n(counts, 5):
        print(str(rank) + " " + pair[0] + " " + str(pair[1]))
        rank = rank + 1

    banner("LENGTHS")
    hist = length_histogram(words)
    for length in sorted(hist.keys()):
        print(str(length) + " " + str(hist[length]))

    banner("OVER")
    for t in [2, 4, 6]:
        print("over" + str(t) + " " + str(count_over_length(words, t)))

    banner("REPEATS")
    print("hapax " + str(hapax_count(counts)))
    mx = max_frequency(counts)
    print("maxfreq " + str(mx))
    leaders = words_with_frequency(counts, mx)
    print("leaders " + str(len(leaders)))
    for w in leaders:
        print("  " + w)


main()
