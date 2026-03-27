"""Logger utility module."""

from abc import ABC, abstractmethod


class LogLevel:
    """Log level constants."""
    DEBUG = 0
    INFO = 1
    WARNING = 2
    ERROR = 3


class BaseLogger(ABC):
    """Abstract base logger."""

    @abstractmethod
    def log(self, level: int, message: str) -> None:
        """Log a message at the given level."""
        ...


class Logger(BaseLogger):
    """Concrete logger implementation."""

    def __init__(self, name: str, level: int = LogLevel.INFO):
        self._name = name
        self._level = level

    def log(self, level: int, message: str) -> None:
        """Log a message at the given level."""
        if level >= self._level:
            print(f"[{self._name}] {message}")

    def info(self, message: str) -> None:
        """Log an info message."""
        self.log(LogLevel.INFO, message)

    def error(self, message: str) -> None:
        """Log an error message."""
        self.log(LogLevel.ERROR, message)

    @classmethod
    def create(cls, name: str) -> "Logger":
        """Factory method to create a logger."""
        cls.validate_name(name)
        return cls(name)

    @classmethod
    def validate_name(cls, name: str) -> None:
        """Validate logger name before creation."""
        if not name:
            raise ValueError("Logger name must not be empty")
