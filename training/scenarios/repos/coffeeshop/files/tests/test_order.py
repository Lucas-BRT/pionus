import pytest

from coffeeshop.config import Config
from coffeeshop.order import Order


def test_total_includes_the_service_fee():
    order = Order(table=1)
    order.add("coffee", 2)
    assert order.total(Config()) == 13.2


def test_negative_quantity_is_refused():
    with pytest.raises(ValueError):
        Order(table=1).add("coffee", -1)


def test_discount_has_a_limit():
    order = Order(table=1, discount=0.9)
    order.add("cake")
    assert order.total(Config()) == round(9 * 1.1 * 0.7, 2)
