#!/usr/bin/python3
import abc
from dataclasses import dataclass


@dataclass
class SendResult:
    is_ok: bool
    http_code: int

class Sender(abc.ABC):
    @abc.abstractmethod
    def send_with_pic(self, text: str, pic) -> SendResult:
        pass

    @abc.abstractmethod
    def send_text(self, text: str, is_markdown: bool) -> SendResult:
        pass
