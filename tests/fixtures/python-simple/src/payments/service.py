"""Payment service — handles payment processing."""

from dataclasses import dataclass
from typing import Optional

from .types import PaymentResult, CardDetails
from ..utils.logger import Logger


class PaymentService:
    """Handles payment processing."""

    def __init__(self, client, logger: Logger):
        self._client = client
        self._logger = logger

    async def process_payment(self, amount: float, user_id: str) -> PaymentResult:
        """Process a payment for the given user."""
        self._logger.info(f"Processing payment for {user_id}")
        return await self._client.charge(amount, user_id)

    def refund(self, tx_id: str, reason: Optional[str] = None) -> bool:
        """Refund a previous transaction."""
        return self._client.refund(tx_id, reason)

    @staticmethod
    def validate_card(card: CardDetails) -> bool:
        """Validate a credit card."""
        return card.number is not None

    @property
    def is_connected(self) -> bool:
        """Check if payment client is connected."""
        return self._client.connected

    def _calculate_fee(self, amount: float) -> float:
        """Internal: calculate processing fee."""
        return amount * 0.029

    def __apply_discount(self, amount: float, code: str) -> float:
        """Name-mangled: apply a discount code."""
        return amount * 0.9
