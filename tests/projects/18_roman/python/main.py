from roman import to_roman, from_roman
from validate import is_valid_roman, safe_decode, safe_encode, in_range


def yn(b):
    if b:
        return "yes"
    return "no"


def main():
    # 1. Encode a spread of integers and round-trip them back.
    print("== encode ==")
    samples = [1, 4, 9, 14, 40, 49, 90, 400, 944, 1994, 2024, 3888, 3999]
    for n in samples:
        r = to_roman(n)
        back = from_roman(r)
        print(f"{n} -> {r} -> {back} rt={yn(back == n)}")

    # 2. Decode known numerals and verify exact expectations.
    print("== decode ==")
    pairs = ["IV", "IX", "XL", "XC", "CD", "CM", "MMXXIV", "MCMXCIV", "MMMCMXCIX"]
    expected = [4, 9, 40, 90, 400, 900, 2024, 1994, 3999]
    i = 0
    while i < len(pairs):
        s = pairs[i]
        v = from_roman(s)
        ok_s = yn(v == expected[i])
        print(f"{s} = {v} ok={ok_s}")
        i = i + 1

    # 3. Validation: well-formed, malformed, out-of-charset, lowercase.
    print("== validate ==")
    cases = ["XIV", "MMXXIV", "IIII", "VV", "IC", "XmI", "", "MMMM", "ABC", "MCMLXXXIV"]
    valid_count = 0
    for c in cases:
        ok_b = is_valid_roman(c)
        if ok_b:
            valid_count = valid_count + 1
        flag = yn(ok_b)
        dec = safe_decode(c)
        print(f"[{c}] valid={flag} decode={dec}")
    print(f"valid_total={valid_count}")

    # 4. Range guards on encoding.
    print("== range ==")
    for n in [0, 1, 3999, 4000, -5]:
        enc = safe_encode(n)
        print(f"{n} inrange={yn(in_range(n))} enc=[{enc}]")

    # 5. Full sweep: every value in [1, 3999] round-trips through both
    #    directions and re-validates. Report the aggregate as integers.
    print("== sweep ==")
    rt_fail = 0
    valid_fail = 0
    total_chars = 0
    n = 1
    while n <= 3999:
        r = to_roman(n)
        total_chars = total_chars + len(r)
        if from_roman(r) != n:
            rt_fail = rt_fail + 1
        if not is_valid_roman(r):
            valid_fail = valid_fail + 1
        n = n + 1
    print(f"roundtrip_failures={rt_fail}")
    print(f"validation_failures={valid_fail}")
    print(f"total_glyphs={total_chars}")

    # 6. Longest canonical numeral length over the domain.
    print("== longest ==")
    longest = 0
    longest_at = 0
    n = 1
    while n <= 3999:
        glyphs = len(to_roman(n))
        if glyphs > longest:
            longest = glyphs
            longest_at = n
        n = n + 1
    print(f"longest_len={longest} at={longest_at} roman={to_roman(longest_at)}")


main()
