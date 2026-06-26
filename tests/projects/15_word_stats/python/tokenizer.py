def normalize(text):
    puncts = [".", ",", ";", ":", "!", "?", "(", ")", "[", "]",
              "\"", "'", "-", "_", "/", "\n", "\t", "*"]
    out = text.lower()
    for p in puncts:
        out = out.replace(p, " ")
    return out


def tokenize(text):
    cleaned = normalize(text)
    raw = cleaned.split(" ")
    words = []
    for w in raw:
        if len(w) > 0:
            words.append(w)
    return words


def sentence_count(text):
    count = 0
    for ch in [".", "!", "?"]:
        count = count + count_char(text, ch)
    return count


def count_char(text, ch):
    n = 0
    parts = text.split(ch)
    n = len(parts) - 1
    return n


def total_chars(words):
    total = 0
    for w in words:
        total = total + len(w)
    return total


def longest(words):
    best = ""
    best_len = -1
    for w in words:
        if len(w) > best_len:
            best_len = len(w)
            best = w
    return best


def shortest(words):
    best = ""
    best_len = 1000000
    for w in words:
        if len(w) < best_len:
            best_len = len(w)
            best = w
    return best
