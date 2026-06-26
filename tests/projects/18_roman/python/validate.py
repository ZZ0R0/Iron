# Validation of Roman numeral strings and of the integer domain.
# A string is a *valid* Roman numeral when it is non-empty, uses only the
# seven Roman letters, decodes to a value in [1, 3999], and is identical to
# the canonical encoding of that value. The canonical round-trip rejects
# malformed forms such as "IIII", "VV", "IC" or out-of-range magnitudes.

from roman import to_roman, from_roman, only_valid_chars

MIN_VALUE = 1
MAX_VALUE = 3999


def in_range(n):
    return n >= MIN_VALUE and n <= MAX_VALUE


def is_valid_roman(s):
    if len(s) == 0:
        return False
    if not only_valid_chars(s):
        return False
    value = from_roman(s)
    if not in_range(value):
        return False
    return to_roman(value) == s


def safe_decode(s):
    # Returns the decoded value when valid, else -1 (a sentinel outside range).
    if is_valid_roman(s):
        return from_roman(s)
    return -1


def safe_encode(n):
    # Returns the canonical numeral when in range, else the empty string.
    if in_range(n):
        return to_roman(n)
    return ""
