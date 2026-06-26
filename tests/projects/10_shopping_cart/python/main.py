from item import Item, make_item
from pricing import QtyRule, round_to_nickel
from cart import Cart, new_cart, checkout


def build_rules():
    rules = []
    rules.append(QtyRule(name="apple", min_qty=10, percent=15))
    rules.append(QtyRule(name="milk", min_qty=3, percent=10))
    rules.append(QtyRule(name="coffee", min_qty=2, percent=20))
    return rules


def run_cart(label, cart, rules):
    print(f"--- cart {label} ---")
    cart.print_lines()
    print(f"units {cart.total_units()}")
    res = checkout(cart, rules, 5000, 5, 825)
    print(f"subtotal {res[0]}")
    print(f"qty_discount {res[1]}")
    print(f"threshold_discount {res[2]}")
    print(f"taxable {res[3]}")
    print(f"tax {res[4]}")
    print(f"total {res[5]}")
    print(f"rounded {round_to_nickel(res[5])}")
    return res[5]


def main():
    rules = build_rules()

    c1 = new_cart()
    c1.add(make_item("apple", 120, 12))
    c1.add(make_item("bread", 250, 2))
    c1.add(make_item("milk", 199, 4))
    c1.add(make_item("coffee", 899, 1))

    c2 = new_cart()
    c2.add(make_item("coffee", 899, 3))
    c2.add(make_item("eggs", 350, 2))
    c2.add(make_item("soap", 450, 1))
    c2.add(make_item("apple", 120, 5))

    n1 = len(c1.items)
    n2 = len(c2.items)
    t1 = run_cart("one", c1, rules)
    t2 = run_cart("two", c2, rules)

    print(f"grand_total {t1 + t2}")
    print(f"carts 2 items {n1 + n2}")

    totals = [t1, t2]
    print(totals)
    print(f"max_total {max(totals)}")
    print(f"min_total {min(totals)}")


main()
