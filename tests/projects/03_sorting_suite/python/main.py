from sorts import bubble_sort, insertion_sort, merge_sort, quick_sort, is_sorted
from datasets import lcg_sequence, reversed_list, sorted_list, all_equal, checksum

def yesno(b):
    if b:
        return "yes"
    return "no"

def report(name, data):
    sb, cb = bubble_sort(data)
    si, ci = insertion_sort(data)
    sm, cm = merge_sort(data)
    sq, cq = quick_sort(data)
    print(f"== {name} (n={len(data)}) ==")
    print(f"merge: {sm}")
    print(f"sorted_ok bubble={yesno(is_sorted(sb))} insertion={yesno(is_sorted(si))} merge={yesno(is_sorted(sm))} quick={yesno(is_sorted(sq))}")
    print(f"cmp bubble={cb} insertion={ci} merge={cm} quick={cq}")
    print(f"checksum={checksum(sm)} agree bubble={yesno(sb == sm)} insertion={yesno(si == sm)} quick={yesno(sq == sm)}")

def main():
    random_data = lcg_sequence(42, 16, 1000)
    rev = reversed_list(12)
    srt = sorted_list(12)
    dup = all_equal(8, 7)
    tiny = [5, 3, 8, 1, 9, 2, 7, 4, 6, 0]

    report("random", random_data)
    report("reversed", rev)
    report("sorted", srt)
    report("duplicates", dup)
    report("tiny", tiny)

    total_cmp = 0
    sizes = [4, 8, 16, 32]
    print("== complexity (merge comparisons) ==")
    for s in sizes:
        seq = lcg_sequence(7, s, 10000)
        sm, cm = merge_sort(seq)
        total_cmp = total_cmp + cm
        print(f"n={s} merge_cmp={cm}")
    print(f"total_merge_cmp={total_cmp}")

    print("== empty and single ==")
    empty = []
    se, ce = quick_sort(empty)
    print(f"empty len={len(se)} cmp={ce}")
    single = [42]
    ss, cs = bubble_sort(single)
    print(f"single: {ss} cmp={cs}")

main()
