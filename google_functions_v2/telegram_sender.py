#!/usr/bin/python3

import sender
import json
import requests
from logger import logger
from dataclasses import dataclass
import typing as t


class TelegramSender(sender.Sender):
    """
    This is a Telegram implementation of class Sender.
    """
    def __init__(self, chat_id: int, bot_id: str):
        self.chat_id = chat_id
        self.bot_id = bot_id


    def try_sending(self, req: requests.Request, type_of_sending: str) -> sender.SendResult:
        prepared = req.prepare()

        response_received: bool = False
        MAX_RETRIES: int = 3
        logger.debug(f'About to send POST request of size {len(prepared.body) / 1024} KiB: request: {req}')
        for i in range(MAX_RETRIES):
            try:
                with requests.Session() as s:
                    response = s.send(prepared, timeout=(1, 19))

            except requests.ConnectTimeout:
                logger.debug(f"Connection timed out")
            except requests.ReadTimeout:
                logger.debug(f"Read timed out")
            except requests.Timeout:
                logger.debug(f"Request timed out")
            else:
                response_received = True
                logger.debug(f'Sent {type_of_sending} to: {self.chat_id}. Response status_code: {response.status_code}, data: "{response.text}"')
                break

        if not response_received:
            return sender.SendResult(is_ok=False, http_code=0, err_str=f"Failed to send HTTP request: {req} {MAX_RETRIES} times")

        try:
            parsed_json = json.loads(response.text)
        except json.JSONDecodeError as exc:
            return sender.SendResult(is_ok=False, http_code=response.status_code,
                                     err_str=f"Failed to decode JSON: {response.text}: details: {type(exc)}, {exc}")
        if 'ok' not in parsed_json:
            return sender.SendResult(is_ok=False, http_code=response.status_code,
                                     err_str=f"'Ok' is not in JSON response: {parsed_json}")
        ok = parsed_json['ok']
        return sender.SendResult(is_ok=ok, http_code=response.status_code,
                                 err_str="")


    def send_text(self, text: str, is_markdown: bool) -> sender.SendResult:
        logger.info(f'text: {text}, is_markdown: {is_markdown}')
        url = f"https://api.telegram.org/bot{self.bot_id}/sendMessage"
        data = {'chat_id': self.chat_id, 'text': text}
        if is_markdown:
            data['parse_mode'] = 'MarkdownV2'
        req = requests.Request('POST', url, data=data)
        result: sender.SendResult = self.try_sending(req, "message")
        logger.info(f"Result: {result}")
        return result


    def send_with_pic(self, text: str, pic) -> sender.SendResult:
        logger.info(f'text: {text}')
        pic.seek(0)
        url = f'https://api.telegram.org/bot{self.bot_id}/sendPhoto'
        form_fields = {
            'chat_id': (None, self.chat_id),
            'photo': ('file.png', pic, 'image/png', {'Content-Type': 'image/png'}),
            'caption': (None, text)
        }
        req = requests.Request('POST', url, files=form_fields)
        result: sender.SendResult = self.try_sending(req, "photo")
        logger.info(f"Result: {result}")
        return result



def get_chat_id_from_update_msg(jsn: str) -> int:
    d = json.loads(jsn)
    return d['message']['chat']['id']
