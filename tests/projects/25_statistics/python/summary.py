# Higher-level analysis built on top of the primitive estimators in stats.py:
# five-number summaries, quartiles, IQR, outlier detection and z-score-like
# deviation counts. Everything stays in integer / centieme space.

from stats import (
    minimum,
    maximum,
    median_x100,
    percentile,
    mean_x100,
    stdev_x100,
    value_range,
)


def quartile1(xs):
    return percentile(xs, 25)


def quartile3(xs):
    return percentile(xs, 75)


def iqr(xs):
    return quartile3(xs) - quartile1(xs)


def five_number(xs):
    lo = minimum(xs)
    q1 = quartile1(xs)
    med = median_x100(xs)
    q3 = quartile3(xs)
    hi = maximum(xs)
    return (lo, q1, med, q3, hi)


def outlier_count(xs):
    q1 = quartile1(xs)
    q3 = quartile3(xs)
    spread = q3 - q1
    low = q1 * 2 - spread * 3
    high = q3 * 2 + spread * 3
    n = 0
    for x in xs:
        v = x * 2
        if v < low or v > high:
            n = n + 1
    return n


def within_one_stdev(xs):
    m = mean_x100(xs)
    sd = stdev_x100(xs)
    n = 0
    for x in xs:
        diff = x * 100 - m
        if diff < 0:
            diff = -diff
        if diff <= sd:
            n = n + 1
    return n


def coefficient_of_variation_x100(xs):
    m = mean_x100(xs)
    if m == 0:
        return 0
    sd = stdev_x100(xs)
    return sd * 10000 // m
