"""Order controller — handles order-related HTTP endpoints."""

from ..payments.service import PaymentService
from ..payments.types import PaymentResult
from ..utils.logger import Logger


class OrderController:
    """Handles order creation and management."""

    def __init__(self, payment_service: PaymentService, logger: Logger):
        self._payment_service = payment_service
        self._logger = logger

    async def create_order(self, user_id: str, amount: float) -> PaymentResult:
        """Create a new order and process payment."""
        self._logger.info(f"Creating order for {user_id}")
        result = await self._payment_service.process_payment(amount, user_id)
        return result

    async def cancel_order(self, order_id: str) -> bool:
        """Cancel an existing order."""
        self._logger.info(f"Cancelling order {order_id}")
        return await self._payment_service.refund(order_id)
