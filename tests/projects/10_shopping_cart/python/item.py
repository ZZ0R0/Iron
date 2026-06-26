from dataclasses import dataclass


@dataclass
class Item:
    name: str
    unit_price: int
    qty: int

    def line_total(self) -> int:
        return self.unit_price * self.qty


def make_item(name, unit_price, qty):
    return Item(name=name, unit_price=unit_price, qty=qty)
