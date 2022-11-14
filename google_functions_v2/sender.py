#!/usr/bin/python3
import abc
from dataclasses import dataclass


@dataclass
class SendResult:
    """Represents status of sending a notification via Sender."""
    is_ok: bool
    http_code: int

class Sender(abc.ABC):
    """Abstract class that responsible for sending text and pic to user.
    This class is helping to implement different types of communication apps that allows us to send notifications, such as email, messenger, sms and etc."""

    @abc.abstractmethod
    def send_with_pic(self, text: str, pic) -> SendResult:
        pass

    @abc.abstractmethod
    def send_text(self, text: str, is_markdown: bool) -> SendResult:
        pass
