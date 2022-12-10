#!/usr/bin/python3
import sender


class StdoutSender(sender.Sender):
    def send_text(self, text: str, is_markdown: bool) -> sender.SendResult:
        print(text)
        return sender.SendResult(is_ok=True, http_code=200, err_str="")

    def send_with_pic(self, text: str, pic) -> sender.SendResult:
        print(text)
        return sender.SendResult(is_ok=True, http_code=200, err_str="")
