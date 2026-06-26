# Bidirectional Roman numeral conversion.
# Encoding uses the standard subtractive-notation table.
# Decoding counts non-overlapping tokens via replace/len so that no
# character indexing is required (keeps the Iron mirror straightforward).

# Full token table in canonical descending order (used for encoding).
VALUES = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1]
SYMBOLS = ["M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I"]

# Two-char subtractive tokens must be consumed before their single-char parts.
SUB_VALUES = [900, 400, 90, 40, 9, 4]
SUB_SYMBOLS = ["CM", "CD", "XC", "XL", "IX", "IV"]
ONE_VALUES = [1000, 500, 100, 50, 10, 5, 1]
ONE_SYMBOLS = ["M", "D", "C", "L", "X", "V", "I"]


def count_token(s, tok):
    after = s.replace(tok, "")
    removed = len(s) - len(after)
    return removed // len(tok)


def to_roman(n):
    result = ""
    i = 0
    while i < len(VALUES):
        v = VALUES[i]
        sym = SYMBOLS[i]
        while n >= v:
            result = result + sym
            n = n - v
        i = i + 1
    return result


def from_roman(s):
    total = 0
    work = s
    i = 0
    while i < len(SUB_SYMBOLS):
        tok = SUB_SYMBOLS[i]
        c = count_token(work, tok)
        total = total + c * SUB_VALUES[i]
        work = work.replace(tok, "")
        i = i + 1
    i = 0
    while i < len(ONE_SYMBOLS):
        tok = ONE_SYMBOLS[i]
        c = count_token(work, tok)
        total = total + c * ONE_VALUES[i]
        work = work.replace(tok, "")
        i = i + 1
    return total


def only_valid_chars(s):
    work = s
    i = 0
    while i < len(ONE_SYMBOLS):
        work = work.replace(ONE_SYMBOLS[i], "")
        i = i + 1
    return len(work) == 0
