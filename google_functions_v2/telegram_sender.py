#!/usr/bin/python3

import sender as s
import json
import requests
from logger import logger




class TelegramSender(s.Sender):
    def __init__(self, chat_id: int, bot_id: str):
        self.chat_id = chat_id
        self.bot_id = bot_id

    def send_text(self, text: str) -> None:
        url = f"https://api.telegram.org/bot{self.bot_id}/sendMessage?chat_id={self.chat_id}&text={text}"
        resp = requests.get(url)
        logger.info(f'status: {resp.status_code}\nheaders: {resp.headers}\nbody: {resp.content}')



def get_chat_id_from_update_msg(jsn: str) -> int:
    d = json.loads(jsn)
    return d["message"]["chat"]["id"]


