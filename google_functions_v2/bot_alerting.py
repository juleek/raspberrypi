#!/usr/bin/python3
import devicedatum as dd
import ingest
import sender as s
import typing as t
import secrets_bot as sec_bot

BOT_NAME: str = "Alerting_bot"
BOT_ID: str = sec_bot.alerting_bot_id

class Alerting(ingest.Consumer):
    def __init__(self, name_min_tuples: t.List[t.Dict[str, float]], sender: s.Sender):
        self.name_min_tuples = name_min_tuples
        self.sender = sender

    def consume(self, datum: dd.DeviceDatum) -> None:
        messages: t.List[str] = []
        for name, min_temp in self.name_min_tuples:
            if name not in datum.name_to_temp:
                continue
            if datum.name_to_temp[name] > min_temp:
                continue
            messages.append(f'"{name}" is {datum.name_to_temp[name]} degrees, which is {min_temp - datum.name_to_temp[name]} degrees lower than threshold {min_temp}!')

        if messages:
            self.sender.send_text("\n".join(messages))
