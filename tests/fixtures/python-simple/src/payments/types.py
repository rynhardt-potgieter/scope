"""Payment types — data classes for payment domain."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class CardDetails:
    """Credit card details."""
    number: Optional[str]
    expiry: str
    cvv: str


@dataclass
class PaymentResult:
    """Result of a payment operation."""
    success: bool
    transaction_id: Optional[str]
    error_message: Optional[str] = None
