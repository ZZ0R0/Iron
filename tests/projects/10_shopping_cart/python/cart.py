from dataclasses import dataclass
from item import Item
from pricing import QtyRule, line_discount, threshold_discount, tax_amount


@dataclass
class Cart:
    items: list

    def add(self, it: Item):
        self.items.append(it)

    def subtotal(self) -> int:
        total = 0
        for it in self.items:
            total = total + it.line_total()
        return total

    def total_units(self) -> int:
        n = 0
        for it in self.items:
            n = n + it.qty
        return n

    def print_lines(self):
        for it in self.items:
            print(f"{it.name} {it.qty} x {it.unit_price} = {it.line_total()}")

    def qty_discount_total(self, rules) -> int:
        total = 0
        for it in self.items:
            total = total + line_discount(it, rules)
        return total


def new_cart() -> Cart:
    return Cart(items=[])


def checkout(cart: Cart, rules, threshold: int, thr_percent: int, rate_bp: int):
    sub = cart.subtotal()
    qty_disc = cart.qty_discount_total(rules)
    after_qty = sub - qty_disc
    thr_disc = threshold_discount(after_qty, threshold, thr_percent)
    taxable = after_qty - thr_disc
    tax = tax_amount(taxable, rate_bp)
    grand = taxable + tax
    result = [sub, qty_disc, thr_disc, taxable, tax, grand]
    return result
