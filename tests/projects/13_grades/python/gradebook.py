# Gradebook analytics over a roster (list of Student): mentions, ranking,
# distribution, per-course statistics, class-wide aggregates, honor roll.

def mention(avg):
    if avg >= 90:
        return "A"
    elif avg >= 80:
        return "B"
    elif avg >= 70:
        return "C"
    elif avg >= 60:
        return "D"
    else:
        return "F"

def ranking(roster):
    # Build (neg_average, name) so a plain ascending sort gives descending
    # average, ties broken by ascending name. Returns sorted list of pairs.
    pairs = []
    for s in roster:
        pairs.append((-s.average(), s.name))
    pairs.sort()
    return pairs

def distribution(roster):
    # Count how many students fall in each mention bucket.
    counts = {}
    counts["A"] = 0
    counts["B"] = 0
    counts["C"] = 0
    counts["D"] = 0
    counts["F"] = 0
    for s in roster:
        m = mention(s.average())
        counts[m] = counts[m] + 1
    return counts

def course_sums(roster):
    sums = {}
    for s in roster:
        i = 0
        while i < len(s.courses):
            c = s.courses[i]
            g = s.grades[i]
            if c in sums:
                sums[c] = sums[c] + g
            else:
                sums[c] = g
            i = i + 1
    return sums

def course_counts(roster):
    cnts = {}
    for s in roster:
        i = 0
        while i < len(s.courses):
            c = s.courses[i]
            if c in cnts:
                cnts[c] = cnts[c] + 1
            else:
                cnts[c] = 1
            i = i + 1
    return cnts

def course_high(roster, course):
    high = -1
    for s in roster:
        g = s.grade_in(course)
        if g > high:
            high = g
    return high

def class_total(roster):
    t = 0
    for s in roster:
        t = t + s.average()
    return t

def class_average(roster):
    n = len(roster)
    if n == 0:
        return 0
    return class_total(roster) // n

def honor_count(roster):
    c = 0
    for s in roster:
        if s.average() >= 90:
            c = c + 1
    return c
