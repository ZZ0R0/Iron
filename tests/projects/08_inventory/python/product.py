from dataclasses import dataclass


@dataclass
class Product:
    sku: int
    name: int
    qty: int
    price: int
    reorder: int

    def value(self) -> int:
        return self.qty * self.price

    def is_low(self) -> int:
        if self.qty <= self.reorder:
            return 1
        return 0

    def is_out(self) -> int:
        if self.qty == 0:
            return 1
        return 0


def make_product(sku: int, name: int, qty: int, price: int, reorder: int) -> Product:
    return Product(sku=sku, name=name, qty=qty, price=price, reorder=reorder)
