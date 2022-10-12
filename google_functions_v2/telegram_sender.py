#!/usr/bin/python3

import sender
import json
import requests
from logger import logger
from dataclasses import dataclass
import typing as t


class TelegramSender(sender.Sender):
    def __init__(self, chat_id: int, bot_id: str):
        self.chat_id = chat_id
        self.bot_id = bot_id


    def processing_of_request(self, req, type_of_sending: str) -> sender.SendResult:
        prepared = req.prepare()

        response_received: bool = False
        MAX_RETRIES: int = 3
        print(f'TelegramBot: About to send POST request of size {len(prepared.body) / 1024} KiB')
        for i in range(MAX_RETRIES):
            try:
                with requests.Session() as s:
                    response = s.send(prepared, timeout=(1, 19))

            except requests.ConnectTimeout:
                logger.warning("Connection timed out")
            except requests.ReadTimeout:
                logger.warning("Read timed out")
            except requests.Timeout:
                logger.warning("Request timed out")
            else:
                response_received = True
                print(f'TelegramBot: Sent {type_of_sending} to: {self.chat_id}. Response status_code: {response.status_code}, data: "{response.text}"')
                break

        if response_received == False:
            return sender.SendResult(is_ok=False, http_code=0)

        try:
            parsed_json = json.loads(response.text)
        except json.JSONDecodeError as exc:
            print('TelegramBot: Failed to decode JSON: {}: {}'.format(type(exc), exc))  # Log Error
            return sender.SendResult(is_ok=True, http_code=response.status_code)
        if 'ok' not in parsed_json:
            return sender.SendResult(is_ok=True, http_code=response.status_code)
        ok = parsed_json['ok']
        return sender.SendResult(is_ok=ok, http_code=response.status_code)


    def send_text(self, text: str, is_markdown: bool) -> sender.SendResult:
        url = f"https://api.telegram.org/bot{self.bot_id}/sendMessage"
        data = {'chat_id': self.chat_id, 'text': text}
        if is_markdown:
            data['parse_mode'] = 'MarkdownV2'
        req = requests.Request('POST', url, data=data)
        return self.processing_of_request(req, "message")


    def send_with_pic(self, text: str, pic) -> sender.SendResult:
        pic.seek(0)
        url = f'https://api.telegram.org/bot{self.bot_id}/sendPhoto'
        form_fields = {
            'chat_id': (None, self.chat_id),
            'photo': ('file.png', pic, 'image/png', {'Content-Type': 'image/png'}),
            'caption': (None, text)
        }
        req = requests.Request('POST', url, files=form_fields)
        return self.processing_of_request(req, "photo")



def get_chat_id_from_update_msg(jsn: str) -> int:
    d = json.loads(jsn)
    return d['message']['chat']['id']
