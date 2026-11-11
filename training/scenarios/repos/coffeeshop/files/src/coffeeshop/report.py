from collections import Counter
from pathlib import Path

from tabulate import tabulate

from coffeeshop.config import Config
from coffeeshop.order import Order

TEMPLATES = Path(__file__).parent.parent.parent / "templates"


def day_summary(orders: list[Order], config: Config) -> str:
    sold: Counter[str] = Counter()
    for order in orders:
        for line in order.lines:
            sold[line.item.name] += line.quantity
    total = sum(order.total(config) for order in orders)
    grid = tabulate(sold.most_common(), headers=["item", "qty"])
    return f"{grid}\n\ntotal: {config.currency} {total:.2f}"


def receipt(order: Order, config: Config) -> str:
    template = (TEMPLATES / "receipt.txt").read_text()
    lines = "\n".join(
        f"{l.quantity}x {l.item.name:<24} {config.currency} {l.subtotal:>6.2f}" for l in order.lines
    )
    return template.format(
        shop=config.shop_name, table=order.table, lines=lines,
        total=f"{config.currency} {order.total(config):.2f}",
    )
