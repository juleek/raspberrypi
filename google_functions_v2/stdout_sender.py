#!/usr/bin/python3
import sender


class StdoutSender(s.Sender):
    def send_text(self, text: str) -> sender.SendResult:
        print(text)
        return sender.SendResult(is_ok=True, http_code=200)

    def send_with_pic(self, text: str, pic) -> sender.SendResult:
        print(text)
        return sender.SendResult(is_ok=True, http_code=200)
