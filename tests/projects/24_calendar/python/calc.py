from date import Date, days_in_month, days_in_year, day_of_year, is_leap


def weekdays():
    return ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]


def zeller(year, month, day):
    y = year
    m = month
    if m < 3:
        m = m + 12
        y = y - 1
    k = y % 100
    j = y // 100
    h = (day + (13 * (m + 1)) // 5 + k + k // 4 + j // 4 + 5 * j) % 7
    return (h + 5) % 7


def weekday_name(year, month, day):
    names = weekdays()
    return names[zeller(year, month, day)]


def is_weekend(year, month, day):
    w = zeller(year, month, day)
    return w >= 5


def day_diff(y1, m1, d1, y2, m2, d2):
    a = Date(year=y1, month=m1, day=d1)
    b = Date(year=y2, month=m2, day=d2)
    return b.ordinal() - a.ordinal()


def add_days(year, month, day, n):
    y = year
    m = month
    d = day + n
    while d > days_in_month(y, m):
        d = d - days_in_month(y, m)
        m = m + 1
        if m > 12:
            m = 1
            y = y + 1
    while d < 1:
        m = m - 1
        if m < 1:
            m = 12
            y = y - 1
        d = d + days_in_month(y, m)
    return Date(year=y, month=m, day=d)


def count_weekends(year, month):
    total = 0
    d = 1
    n = days_in_month(year, month)
    while d <= n:
        if is_weekend(year, month, d):
            total = total + 1
        d = d + 1
    return total


def count_leaps(y1, y2):
    total = 0
    y = y1
    while y <= y2:
        if is_leap(y):
            total = total + 1
        y = y + 1
    return total
