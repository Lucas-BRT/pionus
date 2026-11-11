import sqlite3


class Stock:
    """Quantities in stock, kept in a sqlite database."""

    def __init__(self, path: str) -> None:
        self.connection = sqlite3.connect(path)
        self.connection.execute(
            "create table if not exists stock (code text primary key, quantity int)"
        )

    def quantity(self, code: str) -> int:
        line = self.connection.execute(
            "select quantity from stock where code = ?", (code,)
        ).fetchone()
        return line[0] if line else 0

    def restock(self, code: str, quantity: int) -> None:
        self.connection.execute(
            "insert into stock values (?, ?) "
            "on conflict(code) do update set quantity = quantity + excluded.quantity",
            (code, quantity),
        )
        self.connection.commit()

    def withdraw(self, code: str, quantity: int) -> None:
        current = self.quantity(code)
        if current < quantity:
            raise ValueError(f"insufficent stock of {code}: {current}")
        self.connection.execute(
            "update stock set quantity = ? where code = ?", (current - quantity, code)
        )
        self.connection.commit()
