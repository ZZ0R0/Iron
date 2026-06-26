# A tiny spell checker: given a dictionary of known words, find the closest
# entry to a query using Levenshtein distance. Ties are broken alphabetically
# so the chosen suggestion is fully deterministic.

from distance import levenshtein, lcs_length, common_prefix_len


class Dictionary:
    def __init__(self, words):
        self.words = words

    def size(self):
        return len(self.words)

    def is_known(self, word):
        for w in self.words:
            if w == word:
                return True
        return False

    def min_distance(self, word):
        best = -1
        for w in self.words:
            d = levenshtein(word, w)
            if best < 0 or d < best:
                best = d
        return best

    def best_match(self, word):
        best_word = ""
        best_dist = -1
        for w in self.words:
            d = levenshtein(word, w)
            take = False
            if best_dist < 0:
                take = True
            elif d < best_dist:
                take = True
            elif d == best_dist and w < best_word:
                take = True
            if take:
                best_dist = d
                best_word = w
        return best_word

    def ranked(self, word):
        pairs = []
        for w in self.words:
            d = levenshtein(word, w)
            pairs.append((d, w))
        pairs.sort()
        return pairs

    def suggest(self, word, k):
        pairs = self.ranked(word)
        out = []
        i = 0
        for p in pairs:
            if i >= k:
                break
            out.append(p[1])
            i = i + 1
        return out

    def within(self, word, radius):
        count = 0
        for w in self.words:
            if levenshtein(word, w) <= radius:
                count = count + 1
        return count


def make_dictionary(words):
    d = Dictionary(words)
    return d


def correct_text(d, tokens):
    out = []
    for tok in tokens:
        if d.is_known(tok):
            out.append(tok)
        else:
            out.append(d.best_match(tok))
    return out
