#!/usr/bin/python3

import sender as s
import json
import requests
from logger import logger
from dataclasses import dataclass
import typing as t


@dataclass
class SendResult:
    is_ok: bool
    http_code: int

class TelegramSender(s.Sender):
    def __init__(self, chat_id: int, bot_id: str):
        self.chat_id = chat_id
        self.bot_id = bot_id

    def send_text(self, text: str) -> t.Optional[SendResult]:
        url = f"https://api.telegram.org/bot{self.bot_id}/sendMessage"
        resp = requests.post(url, json={'chat_id': self.chat_id, 'text': text})
        logger.info(f'status: {resp.status_code}, headers: {resp.headers}, body: {resp.content}')

        return SendResult(is_ok=True, http_code=resp.status_code)


    def send_with_pic(self, text: str, pic) -> t.Optional[SendResult]:
        pic.seek(0)
        url = f'https://api.telegram.org/bot{self.bot_id}/sendPhoto'
        form_fields = {
            'chat_id': (None, self.chat_id),
            'photo': ('file.png', pic, 'image/png', {'Content-Type': 'image/png'}),
            'caption': (None, text)
        }
        req = requests.Request('POST', url, files=form_fields)
        prepared = req.prepare()

        response_received: bool = False
        MAX_RETRIES: int = 3
        print(f'TelegramBot: About to send POST request of size {len(prepared.body) / 1024} KiB')
        for i in range(MAX_RETRIES):
            try:
                with requests.Session() as s:
                    response = s.send(prepared, timeout=(1, 19))

            except requests.ConnectTimeout:
                print("Connection timed out")
            except requests.ReadTimeout:
                print("Read timed out")
            except requests.Timeout:
                print("Request timed out")
            else:
                response_received = True
                print(f'TelegramBot: Sent photo to: {self.chat_id}. Response status_code: {response.status_code}, data: "{response.text}"')
                break

        if response_received == False:
            return SendResult(is_ok=False, http_code=0)

        try:
            parsed_json = json.loads(response.text)
        except json.JSONDecodeError as exc:
            print('TelegramBot: Failed to decode JSON: {}: {}'.format(type(exc), exc))  # Log Error
            return None
        if 'ok' not in parsed_json:
            return None
        ok = parsed_json['ok']
        return SendResult(is_ok=ok, http_code=response.status_code)


        # print(f"form_fields = {form_fields}")
        # print(f"prepared.method = {prepared.method}, "
        #       f"prepared.url = {prepared.url}, "
        #       f"k, v in prepared.headers.items() = {prepared.headers.items()}")
        # print(f'TelegramBot: About to send POST request of size {len(prepared.body) / 1024} KiB')



def get_chat_id_from_update_msg(jsn: str) -> int:
    d = json.loads(jsn)
    return d['message']['chat']['id']
