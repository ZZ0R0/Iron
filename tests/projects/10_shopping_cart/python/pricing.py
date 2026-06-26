from dataclasses import dataclass
from item import Item


@dataclass
class QtyRule:
    name: str
    min_qty: int
    percent: int


def line_discount(it: Item, rules) -> int:
    best = 0
    for r in rules:
        if it.name == r.name and it.qty >= r.min_qty:
            d = it.line_total() * r.percent // 100
            if d > best:
                best = d
    return best


def threshold_discount(subtotal: int, threshold: int, percent: int) -> int:
    if subtotal >= threshold:
        return subtotal * percent // 100
    return 0


def tax_amount(taxable: int, rate_bp: int) -> int:
    return taxable * rate_bp // 10000


def round_to_nickel(cents: int) -> int:
    rem = cents % 5
    if rem == 0:
        return cents
    if rem < 3:
        return cents - rem
    return cents + (5 - rem)
