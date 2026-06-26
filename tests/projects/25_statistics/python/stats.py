# Descriptive statistics over integer samples, computed with pure integer
# fixed-point arithmetic so the output is bit-identical to the Rust backend.
# Non-integer results are scaled by 100 ("centiemes").


def isqrt(n):
    if n <= 0:
        return 0
    x = n
    y = (x + 1) // 2
    while y < x:
        x = y
        y = (x + n // x) // 2
    return x


def total(xs):
    s = 0
    for x in xs:
        s = s + x
    return s


def count(xs):
    return len(xs)


def minimum(xs):
    m = xs[0]
    for x in xs:
        if x < m:
            m = x
    return m


def maximum(xs):
    m = xs[0]
    for x in xs:
        if x > m:
            m = x
    return m


def value_range(xs):
    return maximum(xs) - minimum(xs)


def sorted_copy(xs):
    out = []
    for x in xs:
        out.append(x)
    out.sort()
    return out


def mean_x100(xs):
    n = len(xs)
    if n == 0:
        return 0
    return total(xs) * 100 // n


def median_x100(xs):
    n = len(xs)
    if n == 0:
        return 0
    s = sorted_copy(xs)
    mid = n // 2
    if n % 2 == 1:
        return s[mid] * 100
    return (s[mid - 1] + s[mid]) * 50


def sum_squares(xs):
    s = 0
    for x in xs:
        s = s + x * x
    return s


def variance_x100(xs):
    n = len(xs)
    if n == 0:
        return 0
    t = total(xs)
    sq = sum_squares(xs)
    return (n * sq - t * t) * 100 // (n * n)


def stdev_x100(xs):
    n = len(xs)
    if n == 0:
        return 0
    t = total(xs)
    sq = sum_squares(xs)
    scaled = (n * sq - t * t) * 10000 // (n * n)
    return isqrt(scaled)


def frequencies(xs):
    counts = {}
    for x in xs:
        if x in counts:
            counts[x] = counts[x] + 1
        else:
            counts[x] = 1
    return counts


def max_frequency(xs):
    counts = frequencies(xs)
    best = 0
    for k in counts.keys():
        if counts[k] > best:
            best = counts[k]
    return best


def modes(xs):
    counts = frequencies(xs)
    best = max_frequency(xs)
    out = []
    for k in counts.keys():
        if counts[k] == best:
            out.append(k)
    out.sort()
    return out


def percentile(xs, p):
    n = len(xs)
    if n == 0:
        return 0
    s = sorted_copy(xs)
    rank = (p * n + 99) // 100
    if rank < 1:
        rank = 1
    if rank > n:
        rank = n
    return s[rank - 1]
