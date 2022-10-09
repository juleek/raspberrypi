#!/usr/bin/python3
import abc


class Sender(abc.ABC):
    @abc.abstractmethod
    def send_with_pic(self, text: str, pic) -> None:
        pass

    @abc.abstractmethod
    def send_text(self, text: str, is_markdown: bool) -> None:
        pass
