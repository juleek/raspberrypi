import io
import matplotlib.pyplot as mplplt
import matplotlib.dates as mpldates
import numpy as np
import requests
import json
from typing import Set, List, Tuple, Optional
from dataclasses import dataclass, field
from enum import Enum, unique, auto
import big_query as bq
from google.cloud import bigquery
from datetime import datetime
from datetime import timezone
from datetime import timedelta

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

    def send_text_message(self, to_chat_id: int, message: str, parse_mode: ParseMode = None) -> Optional[SendResult]:
        # curl -v -X POST https://api.telegram.org/bot<token>/sendMessage -H
        # "Content-Type: application/json" --data '{"chat_id": -273706948, "text": "test response 3"}'
        url = 'https://api.telegram.org/bot{}/sendMessage'.format(self.token)
        payload = {'chat_id': to_chat_id, "text": message}
        if parse_mode:
            payload['parse_mode'] = parse_mode.name
        headers = {'Content-Type': 'application/json'}
        # print("TelegramBot: About to send_text_message msg: url: {}, payload: {}, headers: {}".
        # format(url, payload, headers))
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

    def send_photo(self, to_chat_id: int, buffer) -> Optional[SendResult]:
        buffer.seek(0)

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

    def send_text_to_all(self, message: str, parse_mode: ParseMode = None):
        if not message:
            return
        self.send_text_to(self.get_list_of_chat_ids(), message=message, parse_mode=parse_mode)

    def send_text_to(self, chat_ids: Set[int], message: str, parse_mode: ParseMode = None) -> None:
        # print('Sending "{}" to chat_ids: {}'.format(message, chat_ids))
        for chat_id in chat_ids:
            # print("BigQueryTelegramBot: chat_id: {}, parse_mode: {}".format(chat_id, parse_mode))
            result: Optional[SendResult] = self.bot.send_text_message(to_chat_id=chat_id,
                                                                      message=message,
                                                                      parse_mode=parse_mode)
            self.__remove_recepient_if_needed(chat_id=chat_id, result=result)

    def send_photo_to_all(self, png_img):
        if not png_img:
            return
        self.send_photo_to(self.get_list_of_chat_ids(), png_img=png_img)

    def send_photo_to(self, chat_ids: Set[int], png_img) -> None:
        for chat_id in chat_ids:
            result: Optional[SendResult] = self.bot.send_photo(to_chat_id=chat_id, buffer=png_img)
            self.__remove_recepient_if_needed(chat_id=chat_id, result=result)

    def __remove_recepient_if_needed(self, chat_id: int, result: Optional[SendResult]) -> None:
        if not result:
            return
        if not result.is_ok and result.http_code == 403:
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
            self.send_text_to(chat_ids={chat_id}, message=msg, parse_mode=ParseMode.Markdown)

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
        self.bot.send_text_to_all(message)

    def alert_chat_ids_if_needed(self,
                                 ambient_temperature: float,
                                 bottom_tube_temperature: float,
                                 chat_ids: Set[int]) -> None:
        message: str = self.__producde_whole_err_msg(ambient_temperature=ambient_temperature,
                                                     bottom_tube_temperature=bottom_tube_temperature)
        self.bot.send_text_to(chat_ids, message)

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
    legend_font_size: int = 19
    format: str = 'png'


def make_plot(plot_info: PlotInfo):
    fig, axes = mplplt.subplots()
    fig.suptitle(plot_info.title, fontsize=plot_info.title_font_size)

    for line in plot_info.lines:
        x: np.ndarray = np.empty(0)
        for qw in line.x:
            x = np.append(x, mpldates.date2num(qw))
        y: np.ndarray = np.sin(x) * 15 + 20 if not line.y.size else line.y
        axes.plot_date(x,
                       y,
                       linestyle='-',
                       color=line.colour,
                       label=line.legend,
                       markersize=3,
                       tz=timezone(timedelta(hours=3)))

    axes.legend(loc='best', fontsize=plot_info.legend_font_size)
    axes.tick_params(labelright=True, labelsize=17)
    mplplt.xticks(rotation=40)
    mplplt.grid(True)

    png_buf: io.BytesIO = io.BytesIO()
    fig.savefig(png_buf, format=plot_info.format, dpi=plot_info.dpi, bbox_inches='tight')
    png_buf.seek(0)

    return fig, axes, png_buf


# noinspection PyShadowingNames,PyShadowingNames,PyShadowingNames
class MonitoringTelegramBot:
    def __init__(self,
                 bot: BigQueryTelegramBot,
                 telemetry_table_id: str,
                 sensor_id_ambient: str,
                 sensor_id_bottom_tube: str,
                 error_string_id: str) -> None:
        self.bot = bot
        self.telemetry_table_id = telemetry_table_id
        self.sensor_id_ambient = sensor_id_ambient
        self.sensor_id_bottom_tube = sensor_id_bottom_tube
        self.error_string_id = error_string_id

    def handle_request(self, parsed_json: json) -> None:
        self.bot.handle_potentially_new_chat_id(parsed_json=parsed_json)

    def fetch_rows(self, column_names: Tuple, interval: timedelta) -> List[Tuple]:
        str_columns: str = ''
        column_names = ("Timestamp",) + column_names
        for str_column in column_names:
            if not str_column:
                continue
            if str_columns:
                str_columns += ', '
            str_columns += str_column
        str_query: str = 'SELECT {} ' \
                         'FROM `{}.{}` ' \
                         'WHERE Timestamp > TIMESTAMP_SUB(CURRENT_TIMESTAMP(), INTERVAL {} SECOND)'. \
            format(str_columns, self.bot.bq.dataset_id, self.telemetry_table_id, int(interval.total_seconds()))

        print('MonitoringTelegramBot: About to execute query: "{}"'.format(str_query))
        job: bigquery.job.QueryJob = self.bot.bq.client.query(str_query, location=self.bot.bq.location)

        result: List[Tuple] = []
        for row in job.result():
            columns: Tuple = ()
            for str_column in column_names:
                if not str_column:
                    columns += (None,)
                else:
                    columns += (row.get(str_column),)
            result.append(columns)

        result.sort(key=lambda x: x[0])
        # for r in result:
        #     print(r)
        return result

    def compose_and_send_digest_to_all(self):
        rows: List[Tuple] = monitoring_bot.fetch_rows(column_names=(monitoring_bot.sensor_id_ambient,
                                                                    monitoring_bot.sensor_id_bottom_tube,
                                                                    monitoring_bot.error_string_id),
                                                      interval=timedelta(hours=24))

        bottom_tube_line: PlotLine = PlotLine(legend="BottomTube", colour=(1, 0, 0))
        ambient_line: PlotLine = PlotLine(legend="Ambient", colour=(0, 0, 1))

        for timestamp, bottom_tube_temp, ambient_temp, _ in rows:
            # print(timestamp, bottom_tube_temp, ambient_temp)
            bottom_tube_line.x.append(timestamp)
            bottom_tube_line.y = np.append(bottom_tube_line.y, bottom_tube_temp)

            ambient_line.x.append(timestamp)
            ambient_line.y = np.append(ambient_line.y, ambient_temp)

        plot_info: PlotInfo = PlotInfo(title='Temp in Tarasovka on {}'.
                                       format(datetime.now().strftime('%m.%d  %H:%M')))

        plot_info.lines.append(bottom_tube_line)
        plot_info.lines.append(ambient_line)
        fig, axes, png_img = make_plot(plot_info)

        # self.bot.bot.send_photo(-208763401, buffer=png_buf)
        self.bot.send_photo_to_all(png_img=png_img)

        # fig.savefig("/home/Void/devel/plot_dpi_300.png", dpi=plot_info.dpi, bbox_inches='tight')
        # mplplt.show()


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
    monitoring_bot_authed_users_table_id: str = "MonitoringBotChats"
    dataset_id = "MainDataSet"
    location = "europe-west2"
    telemetry_sensors_table_id: str = "AllTempSensors"
    sensor_id_bottom_tube: str = "BottomTube"
    sensor_id_ambient: str = "Ambient"
    error_string_id: str = "ErrorString"
    monitoring_bot: MonitoringTelegramBot = MonitoringTelegramBot(
        BigQueryTelegramBot(TelegramBot(token=secrets.monitoring_telegram_bot_token),
                            bq=bq.GBigQuery.wet_run(dataset_id, location),
                            authed_users_table_id=monitoring_bot_authed_users_table_id),
        telemetry_table_id=telemetry_sensors_table_id,
        sensor_id_ambient=sensor_id_ambient,
        sensor_id_bottom_tube=sensor_id_bottom_tube,
        error_string_id=error_string_id)

    monitoring_bot.compose_and_send_digest_to_all()
