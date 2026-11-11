import argparse
import sys

from coffeeshop.config import Config
from coffeeshop.order import Order
from coffeeshop.report import receipt


def main() -> None:
    parser = argparse.ArgumentParser(prog="coffeeshop")
    sub = parser.add_subparsers(dest="command", required=True)
    order = sub.add_parser("order", help="records an order and prints the receipt")
    order.add_argument("items", nargs="+")
    order.add_argument("--table", type=int, default=1)
    order.add_argument("--discount", type=float, default=0.0)
    sub.add_parser("report", help="shows the day summary")  
    args = parser.parse_args()

    config = Config()  
    if args.command == "order":
        created = Order(table=args.table, discount=args.discount)  
        for code in args.items:
            created.add(code)
        print(receipt(created, config))
    else:
        print("no orders recorded today")


if __name__ == "__main__":
    main()
