from bank import Bank
from account import kind_name


def print_statement(bank: Bank, aid: int):
    acc = bank.accounts[aid]
    print(f"-- statement acct {acc.id} ({acc.owner}) --")
    n = acc.history_len()
    for i in range(n):
        k = acc.kinds[i]
        amt = acc.amounts[i]
        print(f"  {kind_name(k)} {amt}")
    print(f"  balance {acc.balance}")


def main():
    bank = Bank(name="Iron Bank", accounts=[], next_id=0, log=[])
    alice = bank.open_account("alice", 10000)
    bob = bank.open_account("bob", 5000)
    carol = bank.open_account("carol", 0)

    print(f"opened {bank.count()} accounts")

    bank.deposit(carol, 7500)
    bank.withdraw(alice, 2500)
    r = bank.withdraw(bob, 999999)
    print(f"overdraw bob ok={r}")

    t1 = bank.transfer(alice, bob, 3000)
    t2 = bank.transfer(bob, carol, 1000)
    t3 = bank.transfer(carol, alice, 500000)
    print(f"transfers {t1} {t2} {t3}")

    gained = bank.apply_interest_all(250)
    print(f"interest paid {gained}")

    print(f"alice balance {bank.balance_of(alice)}")
    print(f"bob balance {bank.balance_of(bob)}")
    print(f"carol balance {bank.balance_of(carol)}")
    print(f"total assets {bank.total_assets()}")
    print(f"richest acct {bank.richest()}")
    print(f"ops logged {bank.ops_logged()}")
    print(f"successful ops {bank.successful_ops()}")

    for aid in range(bank.count()):
        print_statement(bank, aid)


main()
