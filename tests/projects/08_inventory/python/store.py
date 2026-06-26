from dataclasses import dataclass

from product import Product, make_product


@dataclass
class Store:
    skus: list
    products: list
    shipped_total: int
    lost_sales: int

    def add(self, p: Product):
        self.skus.append(p.sku)
        self.products.append(p)

    def find(self, sku: int) -> int:
        i = 0
        while i < len(self.skus):
            if self.skus[i] == sku:
                return i
            i = i + 1
        return -1

    def receive(self, sku: int, n: int):
        idx = self.find(sku)
        if idx >= 0:
            self.products[idx].qty = self.products[idx].qty + n

    def ship(self, sku: int, n: int) -> int:
        idx = self.find(sku)
        if idx < 0:
            return 0
        have = self.products[idx].qty
        if n > have:
            got = have
        else:
            got = n
        self.products[idx].qty = have - got
        self.shipped_total = self.shipped_total + got
        miss = n - got
        if miss > 0:
            self.lost_sales = self.lost_sales + miss
        return got

    def total_value(self) -> int:
        total = 0
        for p in self.products:
            total = total + p.value()
        return total

    def total_units(self) -> int:
        total = 0
        for p in self.products:
            total = total + p.qty
        return total

    def count_out(self) -> int:
        c = 0
        for p in self.products:
            if p.is_out() == 1:
                c = c + 1
        return c

    def low_skus(self) -> list:
        out = []
        for p in self.products:
            if p.is_low() == 1:
                out.append(p.sku)
        out.sort()
        return out

    def top_n(self, k: int) -> list:
        pairs = []
        for p in self.products:
            pairs.append((p.qty, p.sku))
        n = len(pairs)
        i = 0
        while i < n:
            best = i
            j = i + 1
            while j < n:
                if pairs[j][0] > pairs[best][0]:
                    best = j
                elif pairs[j][0] == pairs[best][0] and pairs[j][1] < pairs[best][1]:
                    best = j
                j = j + 1
            tmp = pairs[i]
            pairs[i] = pairs[best]
            pairs[best] = tmp
            i = i + 1
        result = []
        limit = k
        if n < limit:
            limit = n
        i = 0
        while i < limit:
            result.append(pairs[i][1])
            i = i + 1
        return result


def make_store() -> Store:
    return Store(skus=[], products=[], shipped_total=0, lost_sales=0)
