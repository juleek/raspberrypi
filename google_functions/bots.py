import io

from flask import request
import requests
import json
from typing import Set, List, Tuple, Optional
# from collections import namedtuple
from dataclasses import dataclass, field
from enum import Enum, unique, auto
import big_query as bq
from google.cloud import bigquery
from datetime import datetime

import secrets


@dataclass
class SendResult:
    is_ok: bool
    http_code: int


@unique
class ParseMode(Enum):
    Markdown = auto()
    HTML = auto()


class TelegramBot:
    def __init__(self, token: str):
        self.token = token

    def sendTextMessage(self, to_chat_id: int, message: str, parse_mode: ParseMode = None) -> Optional[SendResult]:
        # curl -v -X POST https://api.telegram.org/bot<token>/sendMessage -H
        # "Content-Type: application/json" --data '{"chat_id": -273706948, "text": "test response 3"}'
        url = 'https://api.telegram.org/bot{}/sendMessage'.format(self.token)
        payload = {'chat_id': to_chat_id, "text": message}
        if parse_mode:
            payload['parse_mode'] = parse_mode.name
        headers = {'Content-Type': 'application/json'}
        # print("TelegramBot: About to sendTextMessage msg: url: {}, payload: {}, headers: {}".format(url, payload, headers))
        response = requests.post(url, data=json.dumps(payload), headers=headers)
        print('TelegramBot: Sent message: {}, to: {}. Response status_code: {}, data: "{}"'.
              format(repr(message), to_chat_id, response.status_code, response.text))

        try:
            parsed_json = json.loads(response.text)
        except json.JSONDecodeError as exc:
            print('TelegramBot: Failed to decode JSON: {}: {}'.format(type(exc), exc))  # Log Error
            return None
        if 'ok' not in parsed_json:
            return None
        ok = parsed_json['ok']
        return SendResult(is_ok=ok, http_code=response.status_code)

    def sendPhoto(self, to_chat_id: int, buffer) -> Optional[SendResult]:
        url = 'https://api.telegram.org/bot{}/sendPhoto'.format(self.token)
        form_fields = {
            'chat_id': (None, to_chat_id, None),
            'photo': ('file.png', buffer, 'image/png', {'Content-Type': 'image/png'})
        }
        req = requests.Request('POST', url, files=form_fields)
        prepared = req.prepare()
        # print('{}\n{}\n{}\n\n{}'.format('-----------START-----------',prepared.method + ' ' + prepared.url,
        # '\n'.join('{}: {}'.format(k, v) for k, v in prepared.headers.items()),prepared.body))

        s = requests.Session()
        response = s.send(prepared)
        print('TelegramBot: Sent photo of size {} KiB to: {}. Response status_code: {}, data: "{}"'.
              format(len(prepared.body) / 1024, to_chat_id, response.status_code, response.text))

        try:
            parsed_json = json.loads(response.text)
        except json.JSONDecodeError as exc:
            print('TelegramBot: Failed to decode JSON: {}: {}'.format(type(exc), exc))  # Log Error
            return None
        if 'ok' not in parsed_json:
            return None
        ok = parsed_json['ok']
        return SendResult(is_ok=ok, http_code=response.status_code)

    @staticmethod
    def get_chat_id(parsed_json: json) -> None or int:
        if not parsed_json:
            return None
        try:
            chat_id: int = parsed_json.get("message").get("chat").get("id")
        except BaseException as e:
            print('TelegramBot: Failed to get chat id from: {}: {}'.format(parsed_json, e))  # Log Error
            return None
        return chat_id


class BigQueryTelegramBot:
    # noinspection PyShadowingNames
    def __init__(self, bot: TelegramBot, bq: bq.GBigQuery, authed_users_table_id: str) -> None:
        self.bot = bot
        self.bq = bq
        self.users_table_id = authed_users_table_id
        self.chat_id_column_name = "ChatId"

        if self.bq.dry_run:
            return
        self.table = self.bq.create_table_if_not_created(self.users_table_id)

        schema = [
            bigquery.SchemaField(self.chat_id_column_name, 'INTEGER', mode='REQUIRED')
        ]
        self.table = self.bq.ensure_table_scheme(existing_table=self.table, schema=schema)

        # self.table = self.bq.create_table_if_not_created(self.users_table_id)

    def send_to_all(self, message: str, parse_mode: ParseMode = None):
        if not message:
            return
        self.send_to(self.get_list_of_chat_ids(), message=message, parse_mode=parse_mode)

    def send_to(self, chat_ids: Set[int], message: str, parse_mode: ParseMode = None) -> None:
        # print('Sending "{}" to chat_ids: {}'.format(message, chat_ids))
        for chat_id in chat_ids:
            # print("BigQueryTelegramBot: chat_id: {}, parse_mode: {}".format(chat_id, parse_mode))
            result = self.bot.sendTextMessage(to_chat_id=chat_id, message=message, parse_mode=parse_mode)
            if result and not result.is_ok and result.http_code == 403:
                query: str = 'DELETE FROM `{}.{}` WHERE {} = {};'. \
                    format(self.bq.dataset_id, self.users_table_id, self.chat_id_column_name, chat_id)
                print('Result of sending a message to user/chat {} is "{}" => deleting the chat_id from BigQuery: {}'.
                      format(chat_id, result, query))
                if not self.bq.dry_run:
                    self.bq.client.query(query=query)

    def get_list_of_chat_ids(self) -> Set[int]:
        if self.bq.dry_run:
            return set()

        result: Set[int] = set()
        existing_ids: bigquery.table.RowIterator = self.bq.client.list_rows(self.table)
        for row in existing_ids:
            result.add(row.get(self.chat_id_column_name))
        return result

    def handle_potentially_new_chat_id(self, parsed_json: json) -> None:
        chat_id = TelegramBot.get_chat_id(parsed_json=parsed_json)
        if not chat_id:
            return
        # print('Successfully got chat_id: {}'.format(chat_id))
        existing_ids: Set[int] = self.get_list_of_chat_ids()

        if chat_id in existing_ids:
            print('There already exists such id: {} in table: "{}" => not adding it'.
                  format(chat_id, self.table.table_id))
        else:
            # Since we have not implemented proper auth method, we do not allow any ID to be inserted into
            # big query, instead we will ask user to insert it manually

            # print('There is no such id: {} in table: "{}" => adding it'.format(chat_id, self.bot.table.table_id))
            # self.bot.bq.insert_rows(self.bot.table, [(chat_id,)])

            msg: str = 'Authenticating has not been implemented yet, so insert your chat id into ' \
                       'Google BigQuery manually by issuing:\n' \
                       '```\n' \
                       'INSERT INTO `{}.{}` ({}) VALUES ({});\n' \
                       '```' \
                       'at https://console.cloud.google.com/bigquery'. \
                format(self.bq.dataset_id, self.users_table_id, self.chat_id_column_name, chat_id)
            self.send_to(chat_ids={chat_id}, message=msg, parse_mode=ParseMode.Markdown)

        # for row in existing_ids:
        # print('{}, {}, type: {}'.format(row.keys(), row.values(), type(row)))
        # print(row.get(self.chat_id_column_name), type(row.get("ChatId")))


class AlertingTelegramBot:
    def __init__(self,
                 bot: BigQueryTelegramBot,
                 ambient_temp_threshold: float,
                 bottom_tube_temp_threshold: float):
        self.bot = bot
        self.ambient_temp_threshold = ambient_temp_threshold
        self.bottom_tube_temp_threshold = bottom_tube_temp_threshold

    def alert_all_if_needed(self, ambient_temperature: float, bottom_tube_temperature: float) -> None:
        message: str = self.__producde_whole_err_msg(ambient_temperature=ambient_temperature,
                                                     bottom_tube_temperature=bottom_tube_temperature)
        # print('AlertingBot: msg: {}'.format(message))
        self.bot.send_to_all(message)

    def alert_chat_ids_if_needed(self,
                                 ambient_temperature: float,
                                 bottom_tube_temperature: float,
                                 chat_ids: Set[int]) -> None:
        message: str = self.__producde_whole_err_msg(ambient_temperature=ambient_temperature,
                                                     bottom_tube_temperature=bottom_tube_temperature)
        self.bot.send_to(chat_ids, message)

    @staticmethod
    def __produce_err_msg(sensor_name: str, current: float, threshold: float) -> str:
        if current < threshold:
            return '{} temperature is {} degrees, which is {} degrees lower than threshold {}!'. \
                format(sensor_name, current, threshold - current, threshold)
        return ''

    def __producde_whole_err_msg(self, ambient_temperature: float, bottom_tube_temperature: float) -> str:
        message: str = '\n'.join(filter(None, [
            self.__produce_err_msg("Ambient", current=ambient_temperature, threshold=self.ambient_temp_threshold),
            self.__produce_err_msg("BottomTube", current=bottom_tube_temperature,
                                   threshold=self.bottom_tube_temp_threshold)]))
        return message

    def handle_request(self, parsed_json: json) -> None:
        self.bot.handle_potentially_new_chat_id(parsed_json=parsed_json)


# https://googleapis.github.io/google-cloud-python/latest/bigquery/generated/google.cloud.bigquery.client.Client.html#google.cloud.bigquery.client.Client.query
# SELECT * FROM `MainDataSet.AlertingBotChats`
# DELETE FROM `MainDataSet.AlertingBotChats` WHERE ChatId = 23464524;
# INSERT INTO `MainDataSet.AlertingBotChats` (ChatId) VALUES (23464524);


# def on_telegram_http_request(request: request) -> None:
#     # curl -X POST "https://europe-west1-tarasovka-monitoring.cloudfunctions.net/on_telegram_http_request"
#     # -H "Content-Type:application/json" --data '{"name":"Keyboard Cat"}'
#     # <Request 'http://europe-west1-tarasovka-monitoring.cloudfunctions.net' [POST]>
#     print('Got request:{}'.format(request))
#     print('Data:\n{}'.format(request.data))


import matplotlib.pyplot as mplplt
import matplotlib.dates as mpldates
import numpy as np


@dataclass
class PlotLine:
    legend: str
    colour: Tuple[float, float, float]
    x: List[datetime] = field(default_factory=list)
    y: np.ndarray = np.empty(0)


@dataclass
class PlotInfo:
    title: str
    lines: List[PlotLine] = field(default_factory=list)
    dpi: int = 500
    title_font_size: int = 23
    legend_font_size: int = 18
    format: str = 'png'


@dataclass
class Test:
    a: int


def make_plot(plot_info: PlotInfo):
    fig, axes = mplplt.subplots()
    fig.suptitle(plot_info.title, fontsize=plot_info.title_font_size)

    for line in plot_info.lines:
        x: np.ndarray = np.empty(0)
        for qw in line.x:
            x = np.append(x, mpldates.date2num(qw))
        y: np.ndarray = np.sin(x) * 15 + 20 if not line.y else line.y
        axes.plot_date(x, y, linestyle='-', color=line.colour, label=line.legend, antialiased=True)

    axes.legend(loc='best', fontsize=plot_info.legend_font_size)
    mplplt.xticks(rotation=40)

    png_buf: io.BytesIO = io.BytesIO()
    fig.savefig(png_buf, format=plot_info.format, dpi=plot_info.dpi, bbox_inches='tight')
    png_buf.seek(0)

    return fig, axes, png_buf


class MonitoringTelegramBot:
    def __init__(self, bot: BigQueryTelegramBot) -> None:
        self.bot = bot

    def handle_request(self, parsed_json: json) -> None:
        self.bot.handle_potentially_new_chat_id(parsed_json=parsed_json)


if __name__ == "__main__":
    print("asdf 1")
    # ================================================================================================
    # bq_alerting_telegram_bot: BigQueryTelegramBot = BigQueryTelegramBot(
    #     bot=TelegramBot(secrets.monitoring_telegram_bot_token),
    #     bq=bq.GBigQuery(dataset_id="MainDataSet", location='europe-west2', dry_run=False),
    #     authed_users_table_id="AlertingBotChats")
    # bq_alerting_telegram_bot.bq.delete_table("AlertingBotChats")
    # bq_alerting_telegram_bot.handle_request(json.loads(b'{"message":{"chat":{"id":132}}}'))

    # ================================================================================================
    # alerting_bot_authed_users_table_id = "AlertingBotChats"
    # dataset_id = "MainDataSet"
    # location = "europe-west2"
    # alerting_bot: AlertingTelegramBot = AlertingTelegramBot(
    #     bots.BigQueryTelegramBot(bots.TelegramBot(token=secrets.monitoring_telegram_bot_token),
    #                              bq=bq.GBigQuery.wet_run(dataset_id, location),
    #                              authed_users_table_id=alerting_bot_authed_users_table_id),
    #     ambient_temp_threshold=6,
    #     bottom_tube_temp_threshold=12)
    # alerting_bot.alert_all_if_needed(ambient_temperature=5, bottom_tube_temperature=10)

    dates: List[str] = ["2018-11-15 12:06:08.610715 UTC", "2018-11-16 03:35:39.287725 UTC",
                        "2018-11-16 03:55:39.287725 UTC", "2018-11-16 04:20:39.287725 UTC",
                        "2018-11-16 05:50:39.287725 UTC", "2018-11-16 06:10:39.287725 UTC",
                        "2018-11-16 06:40:39.287725 UTC", "2018-11-16 07:40:39.287725 UTC",
                        "2018-11-16 08:40:39.287725 UTC", "2018-11-16 10:40:39.287725 UTC",
                        "2018-11-16 12:40:39.287725 UTC", "2018-11-16 14:40:39.287725 UTC",
                        "2018-11-16 16:40:39.287725 UTC", "2018-11-16 18:40:39.287725 UTC",
                        "2018-11-16 20:40:39.287725 UTC", "2018-11-16 23:40:39.287725 UTC",
                        "2018-11-17 03:40:39.287725 UTC", "2018-11-17 13:40:39.287725 UTC",
                        "2018-11-18 04:40:39.287725 UTC", "2018-11-18 18:40:39.287725 UTC",
                        "2018-11-19 03:40:39.287725 UTC", "2018-11-19 15:40:39.287725 UTC"
                        ]

    # monitoring_bot_authed_users_table_id: str = "MonitoringBotChats"
    # dataset_id = "MainDataSet"
    # location = "europe-west2"
    # monitoring_bot: MonitoringTelegramBot = MonitoringTelegramBot(
    #     BigQueryTelegramBot(TelegramBot(token=secrets.monitoring_telegram_bot_token),
    #                         bq=bq.GBigQuery.wet_run(dataset_id, location),
    #                         authed_users_table_id=monitoring_bot_authed_users_table_id))

    plot_info: PlotInfo = PlotInfo(title='Temperature in Tarasovka for 12.04')
    bottom_tube_line: PlotLine = PlotLine(legend="BottomTube", colour=(1, 0, 0))
    for date in dates:  # date looks like: "2018-11-16 02:06:08.610715 UTC"
        stripped, _, _ = date.rpartition(' UTC')
        bottom_tube_line.x.append(datetime.strptime(stripped, '%Y-%m-%d %H:%M:%S.%f'))

    plot_info.lines.append(bottom_tube_line)


    fig, axes, png_buf = make_plot(plot_info)

    bot: TelegramBot = TelegramBot(token=secrets.monitoring_telegram_bot_token)
    # bot.sendPhoto(23464524, buffer=png_buf)

    fig.savefig("/home/Void/devel/plot_dpi_300.png", dpi=plot_info.dpi, bbox_inches='tight')
    mplplt.show()
