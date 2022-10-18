#!/usr/bin/python3
import abc
from dataclasses import dataclass


@dataclass
class SendResult:
    """Class for keeping information about the result of receiving response."""
    is_ok: bool
    http_code: int

class Sender(abc.ABC):
    """Abstract class that has methods that we need for sending notifications to user.
    This class will help to implement different types of communication apps that allows to send notifications, such as email, messenger and etc."""

    @abc.abstractmethod
    def send_with_pic(self, text: str, pic) -> SendResult:
        pass

    @abc.abstractmethod
    def send_text(self, text: str, is_markdown: bool) -> SendResult:
        pass
