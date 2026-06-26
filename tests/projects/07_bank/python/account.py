from dataclasses import dataclass

# transaction kind codes: 1=deposit 2=withdraw 3=interest 4=transfer_in 5=transfer_out

@dataclass
class Account:
    id: int
    owner: str
    balance: int
    kinds: list
    amounts: list

    def deposit(self, amount: int) -> int:
        if amount <= 0:
            return 0
        self.balance = self.balance + amount
        self.kinds.append(1)
        self.amounts.append(amount)
        return 1

    def withdraw(self, amount: int) -> int:
        if amount <= 0:
            return 0
        if amount > self.balance:
            return 0
        self.balance = self.balance - amount
        self.kinds.append(2)
        self.amounts.append(amount)
        return 1

    def credit(self, amount: int, kind: int) -> int:
        self.balance = self.balance + amount
        self.kinds.append(kind)
        self.amounts.append(amount)
        return 1

    def debit(self, amount: int, kind: int) -> int:
        if amount > self.balance:
            return 0
        self.balance = self.balance - amount
        self.kinds.append(kind)
        self.amounts.append(amount)
        return 1

    def apply_interest(self, rate_bps: int) -> int:
        gain = self.balance * rate_bps // 10000
        if gain <= 0:
            return 0
        self.balance = self.balance + gain
        self.kinds.append(3)
        self.amounts.append(gain)
        return gain

    def history_len(self) -> int:
        return len(self.kinds)


def kind_name(kind: int) -> str:
    if kind == 1:
        return "deposit"
    if kind == 2:
        return "withdraw"
    if kind == 3:
        return "interest"
    if kind == 4:
        return "transfer_in"
    if kind == 5:
        return "transfer_out"
    return "unknown"
