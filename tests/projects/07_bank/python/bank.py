from dataclasses import dataclass
from account import Account

# transaction kind codes: 4=transfer_in 5=transfer_out


@dataclass
class Bank:
    name: str
    accounts: list
    next_id: int
    log: list

    def open_account(self, owner: str, opening: int) -> int:
        aid = self.next_id
        acc = Account(id=aid, owner=owner, balance=0, kinds=[], amounts=[])
        if opening > 0:
            acc.deposit(opening)
        self.accounts.append(acc)
        self.next_id = self.next_id + 1
        return aid

    def deposit(self, aid: int, amount: int) -> int:
        ok = self.accounts[aid].deposit(amount)
        self.log.append(ok)
        return ok

    def withdraw(self, aid: int, amount: int) -> int:
        ok = self.accounts[aid].withdraw(amount)
        self.log.append(ok)
        return ok

    def transfer(self, src: int, dst: int, amount: int) -> int:
        if amount <= 0:
            self.log.append(0)
            return 0
        ok = self.accounts[src].debit(amount, 5)
        if ok == 0:
            self.log.append(0)
            return 0
        self.accounts[dst].credit(amount, 4)
        self.log.append(1)
        return 1

    def apply_interest_all(self, rate_bps: int) -> int:
        total = 0
        for i in range(len(self.accounts)):
            total = total + self.accounts[i].apply_interest(rate_bps)
        self.log.append(total)
        return total

    def balance_of(self, aid: int) -> int:
        return self.accounts[aid].balance

    def total_assets(self) -> int:
        total = 0
        for i in range(len(self.accounts)):
            total = total + self.accounts[i].balance
        return total

    def count(self) -> int:
        return len(self.accounts)

    def ops_logged(self) -> int:
        return len(self.log)

    def successful_ops(self) -> int:
        c = 0
        for i in range(len(self.log)):
            if self.log[i] > 0:
                c = c + 1
        return c

    def richest(self) -> int:
        best = 0
        best_bal = -1
        for i in range(len(self.accounts)):
            bal = self.accounts[i].balance
            if bal > best_bal:
                best_bal = bal
                best = self.accounts[i].id
        return best
