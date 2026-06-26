# Query layer over a list[Record]: filtering, group-by on a string key, and
# integer aggregates (count, sum, average, min, max). Results come back as
# dicts keyed by the group label so callers can sort and print deterministically.

from parse import Record, field_of, number_of


def filter_salary_min(records, threshold):
    out = []
    for r in records:
        if r.salary >= threshold:
            out.append(r)
    return out


def filter_age_below(records, threshold):
    out = []
    for r in records:
        if r.age < threshold:
            out.append(r)
    return out


def filter_dept(records, dept):
    out = []
    for r in records:
        if r.dept == dept:
            out.append(r)
    return out


def count_by(records, key):
    counts = {}
    for r in records:
        g = field_of(r, key)
        if g in counts:
            counts[g] = counts[g] + 1
        else:
            counts[g] = 1
    return counts


def sum_by(records, key, value_key):
    sums = {}
    for r in records:
        g = field_of(r, key)
        v = number_of(r, value_key)
        if g in sums:
            sums[g] = sums[g] + v
        else:
            sums[g] = v
    return sums


def avg_by(records, key, value_key):
    sums = sum_by(records, key, value_key)
    counts = count_by(records, key)
    avgs = {}
    for g in sums:
        avgs[g] = sums[g] // counts[g]
    return avgs


def max_by(records, key, value_key):
    best = {}
    for r in records:
        g = field_of(r, key)
        v = number_of(r, value_key)
        if g in best:
            if v > best[g]:
                best[g] = v
        else:
            best[g] = v
    return best


def min_by(records, key, value_key):
    best = {}
    for r in records:
        g = field_of(r, key)
        v = number_of(r, value_key)
        if g in best:
            if v < best[g]:
                best[g] = v
        else:
            best[g] = v
    return best


def total_salary(records):
    t = 0
    for r in records:
        t = t + r.salary
    return t


def overall_average_salary(records):
    n = len(records)
    if n == 0:
        return 0
    return total_salary(records) // n
