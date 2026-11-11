from dataclasses import dataclass


@dataclass(frozen=True)
class Item:
    code: str
    name: str
    price: float
    category: str


MENU = {
    "coffee": Item("coffee", "Brewed coffee", 6.00, "drink"),
    "espresso": Item("espresso", "Espresso", 7.50, "drink"),
    "cappuccino": Item("cappuccino", "Cappuccino", 11.00, "drink"),
    "tea": Item("tea", "Chamomile tea", 5.50, "drink"),
    "cheese-bread": Item("cheese-bread", "Cheese bread", 4.50, "savory"),
    "croquette": Item("croquette", "Croquette", 8.00, "savory"),
    "cake": Item("cake", "Slice of carrot cake", 9.00, "sweet"),
}


def find(code: str) -> Item:
    try:
        return MENU[code]
    except KeyError:
        raise ValueError(f"unknown item: {code}") from None


def by_category(category: str) -> list[Item]:
    return [item for item in MENU.values() if item.category == category]
