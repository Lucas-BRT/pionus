from dataclasses import dataclass, field

from coffeeshop.menu import Item, find
from coffeeshop.config import Config


@dataclass
class Line:
    item: Item
    quantity: int = 1

    @property
    def subtotal(self) -> float:
        return self.item.price * self.quantity


@dataclass
class Order:
    table: int
    lines: list[Line] = field(default_factory=list)
    discount: float = 0.0

    def add(self, code: str, quantity: int = 1) -> None:
        if quantity <= 0:
            raise ValueError("quantity must be positive")
        for line in self.lines:
            if line.item.code == code:
                line.quantity += quantity
                return
        self.lines.append(Line(find(code), quantity))

    def remove(self, code: str) -> None:
        self.lines = [l for l in self.lines if l.item.code != code]

    def subtotal(self) -> float:
        return sum(line.subtotal for line in self.lines)

    def total(self, config: Config) -> float:
        bruto = self.subtotal() * (1 + config.service_fee)
        discount = min(self.discount, config.max_discount)
        return round(bruto * (1 - discount), 2)
