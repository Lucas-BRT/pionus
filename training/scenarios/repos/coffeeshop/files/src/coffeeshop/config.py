from dataclasses import dataclass


@dataclass
class Config:
    shop_name: str = "Corner Coffee"
    currency: str = "$"
    service_fee: float = 0.10
    max_discount: float = 0.30
    stock_file: str = "stock.sqlite"
