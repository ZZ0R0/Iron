# The vending machine itself: a small finite state machine.
# Products live in three parallel lists (id, price in cents, stock).

from state import (
    ST_IDLE, ST_COLLECTING, ST_DISPENSING,
    is_valid_coin, make_change, state_name,
)


class Machine:
    state: int
    balance: int
    selected: int
    vended: int
    rejected: int
    revenue: int
    ids: list
    prices: list
    stock: list

    def __init__(self):
        self.state = ST_IDLE
        self.balance = 0
        self.selected = -1
        self.vended = 0
        self.rejected = 0
        self.revenue = 0
        self.ids = []
        self.prices = []
        self.stock = []

    def add_product(self, pid: int, price: int, qty: int):
        self.ids.append(pid)
        self.prices.append(price)
        self.stock.append(qty)

    def slot_of(self, pid: int) -> int:
        for i in range(len(self.ids)):
            if self.ids[i] == pid:
                return i
        return -1

    def price_of(self, pid: int) -> int:
        i = self.slot_of(pid)
        if i < 0:
            return -1
        return self.prices[i]

    def stock_of(self, pid: int) -> int:
        i = self.slot_of(pid)
        if i < 0:
            return -1
        return self.stock[i]

    # Insert a coin. Invalid coins are rejected and counted.
    def insert(self, coin: int) -> int:
        if is_valid_coin(coin) == 0:
            self.rejected = self.rejected + 1
            return 0
        self.balance = self.balance + coin
        if self.state == ST_IDLE:
            self.state = ST_COLLECTING
        return 1

    # Status: 2 = dispensed, 1 = need more money, 0 = out of stock / bad id.
    def select(self, pid: int) -> int:
        i = self.slot_of(pid)
        if i < 0:
            return 0
        if self.stock[i] <= 0:
            return 0
        if self.balance < self.prices[i]:
            self.selected = pid
            return 1
        self.selected = pid
        self.state = ST_DISPENSING
        return 2

    # Deduct price, drop stock, bank revenue, return change in cents.
    def dispense(self) -> int:
        i = self.slot_of(self.selected)
        price = self.prices[i]
        self.stock[i] = self.stock[i] - 1
        self.revenue = self.revenue + price
        change = self.balance - price
        self.balance = 0
        self.vended = self.vended + 1
        self.selected = -1
        self.state = ST_IDLE
        return change

    # Cancel: refund whatever is in the machine, return the amount.
    def cancel(self) -> int:
        refund = self.balance
        self.balance = 0
        self.selected = -1
        self.state = ST_IDLE
        return refund

    def total_stock(self) -> int:
        s = 0
        for q in self.stock:
            s = s + q
        return s

    def show_state(self):
        print(f"state={state_name(self.state)} balance={self.balance}")

    # Drive a full purchase: feed coins, then select, dispense if possible.
    def buy(self, pid: int, inserted: list):
        print(f"-- buy product {pid} --")
        for c in inserted:
            ok = self.insert(c)
            tag = "ok"
            if ok != 1:
                tag = "rejected"
            print(f"insert {c}: {tag} -> state={state_name(self.state)} balance={self.balance}")
        status = self.select(pid)
        if status == 2:
            change = self.dispense()
            coinsout = make_change(change)
            print(f"select {pid}: dispensed change={change} coins={coinsout}")
            print(f"  state={state_name(self.state)} balance={self.balance}")
        elif status == 1:
            need = self.price_of(pid) - self.balance
            print(f"select {pid}: need {need} more, state={state_name(self.state)}")
        else:
            print(f"select {pid}: unavailable, state={state_name(self.state)}")
