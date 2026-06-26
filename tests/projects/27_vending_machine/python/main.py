from state import make_change, change_count, is_valid_coin
from machine import Machine
from state import state_name


def build_machine() -> Machine:
    m = Machine()
    m.add_product(1, 150, 2)   # water
    m.add_product(2, 125, 1)   # cola
    m.add_product(3, 200, 3)   # juice
    m.add_product(4, 75, 0)    # sold-out snack
    return m


def tag_ok(ok: int) -> str:
    if ok == 1:
        return "ok"
    return "rejected"


def main():
    print("=== Change making ===")
    for amount in [0, 5, 30, 75, 99, 140, 285, 365]:
        cs = make_change(amount)
        print(f"change {amount} -> {cs} count {change_count(amount)}")

    print("=== Coin validation ===")
    for c in [5, 7, 25, 30, 200, 201]:
        v = is_valid_coin(c)
        tag = "no"
        if v == 1:
            tag = "yes"
        print(f"coin {c} valid {tag}")

    print("=== Scenarios ===")
    m = build_machine()
    m.show_state()

    # 1) water 150, pay 100+50 -> no change
    m.buy(1, [100, 50])

    # 2) juice 200, pay 200+100 -> change 100
    m.buy(3, [200, 100])

    # 3) cola 125, pay 100 (short by 25), then add 50 and finish
    print("-- staged buy product 2 --")
    ok = m.insert(100)
    print(f"insert 100: {tag_ok(ok)} balance={m.balance}")
    s = m.select(2)
    print(f"select 2 attempt1 status {s} state={state_name(m.state)}")
    ok = m.insert(50)
    print(f"insert 50: {tag_ok(ok)} balance={m.balance}")
    s = m.select(2)
    if s == 2:
        change = m.dispense()
        cs = make_change(change)
        print(f"select 2 attempt2 dispensed change={change} coins={cs}")
    m.show_state()

    # 4) invalid coin then cancel
    print("-- invalid + cancel --")
    ok = m.insert(7)
    print(f"insert 7: {tag_ok(ok)} balance={m.balance}")
    ok = m.insert(100)
    print(f"insert 100: {tag_ok(ok)} balance={m.balance}")
    refund = m.cancel()
    cs = make_change(refund)
    print(f"cancel: refund={refund} coins={cs} state={state_name(m.state)}")

    # 5) out of stock (snack id 4, stock 0)
    m.buy(4, [100])
    refund = m.cancel()
    print(f"cleanup refund {refund}")

    # 6) cola already depleted in scenario 3 -> unavailable
    m.buy(2, [200])

    print("=== Summary ===")
    print(f"vended {m.vended}")
    print(f"rejected {m.rejected}")
    print(f"revenue {m.revenue}")
    print(f"remaining stock {m.total_stock()}")
    for pid in [1, 2, 3, 4]:
        print(f"product {pid} price {m.price_of(pid)} stock {m.stock_of(pid)}")


main()
