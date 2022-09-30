#!/usr/bin/python3

import sender as s



class StdoutSender(s.Sender):
    def send_text(self, text: str) -> None:
        print(text)

    def send_with_pic(self, text: str, pic) -> None:
        pass
