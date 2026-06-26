from dataclasses import dataclass


def month_days():
    return [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]


def is_leap(year):
    if year % 400 == 0:
        return True
    if year % 100 == 0:
        return False
    return year % 4 == 0


def days_in_month(year, month):
    if month == 2 and is_leap(year):
        return 29
    md = month_days()
    return md[month - 1]


def days_in_year(year):
    if is_leap(year):
        return 366
    return 365


def day_of_year(year, month, day):
    total = 0
    m = 1
    while m < month:
        total = total + days_in_month(year, m)
        m = m + 1
    return total + day


def is_valid(year, month, day):
    if month < 1 or month > 12:
        return False
    if day < 1:
        return False
    if day > days_in_month(year, month):
        return False
    return True


@dataclass
class Date:
    year: int
    month: int
    day: int

    def ordinal(self):
        total = 0
        y = 1
        while y < self.year:
            total = total + days_in_year(y)
            y = y + 1
        return total + day_of_year(self.year, self.month, self.day)

    def label(self):
        return self.year * 10000 + self.month * 100 + self.day
