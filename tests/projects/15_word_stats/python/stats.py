def frequencies(words):
    counts = {}
    for w in words:
        if w in counts:
            counts[w] = counts[w] + 1
        else:
            counts[w] = 1
    return counts


def unique_count(counts):
    return len(counts.keys())


def top_n(counts, n):
    pairs = []
    for k in counts.keys():
        pairs.append((-counts[k], k))
    pairs.sort()
    out = []
    i = 0
    for p in pairs:
        if i >= n:
            break
        out.append((p[1], -p[0]))
        i = i + 1
    return out


def avg_word_len_x100(words):
    if len(words) == 0:
        return 0
    total = 0
    for w in words:
        total = total + len(w)
    return total * 100 // len(words)


def count_over_length(words, threshold):
    n = 0
    for w in words:
        if len(w) > threshold:
            n = n + 1
    return n


def length_histogram(words):
    hist = {}
    for w in words:
        length = len(w)
        if length in hist:
            hist[length] = hist[length] + 1
        else:
            hist[length] = 1
    return hist


def hapax_count(counts):
    n = 0
    for k in counts.keys():
        if counts[k] == 1:
            n = n + 1
    return n


def max_frequency(counts):
    best = 0
    for k in counts.keys():
        if counts[k] > best:
            best = counts[k]
    return best


def words_with_frequency(counts, freq):
    out = []
    for k in counts.keys():
        if counts[k] == freq:
            out.append(k)
    out.sort()
    return out
