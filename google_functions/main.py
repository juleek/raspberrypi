from pathlib import Path
import sys
import flask
import pytz
from flask import request

import telemetry_processor as tp
import big_query as bq
import bots
import secrets

# ========================================================
# CONSTANTS

location: str = "europe-west2"
dataset_id: str = "MainDataSet"

telemetry_sensors_table_id: str = "AllTempSensors"
sensor_id_bottom_tube: str = "BottomTube"
sensor_id_ambient: str = "Ambient"
error_string_id: str = "ErrorString"

ambient_alert_temperature: float = 6
bottom_tube_alert_temperature: float = 12
tz = pytz.timezone("Europe/Moscow")

alerting_bot_authed_users_table_id: str = "AlertingBotChats"
monitoring_bot_authed_users_table_id: str = "MonitoringBotChats"

# ========================================================
# INIT COMPONENTS

google_big_query_global: bq.GBigQuery = bq.GBigQuery(dataset_id=dataset_id,
                                                     location=location,
                                                     dry_run=False)

alerting_telegram_bot: bots.AlertingTelegramBot = bots.AlertingTelegramBot(
    bot=bots.BigQueryTelegramBot(bot=bots.TelegramBot(secrets.alerting_telegram_bot_token),
                                 bq=google_big_query_global,
                                 authed_users_table_id=alerting_bot_authed_users_table_id),
    ambient_temp_threshold=ambient_alert_temperature,
    bottom_tube_temp_threshold=bottom_tube_alert_temperature)

monitoring_telegram_bot: bots.MonitoringTelegramBot = bots.MonitoringTelegramBot(
    bot=bots.BigQueryTelegramBot(bot=bots.TelegramBot(secrets.monitoring_telegram_bot_token),
                                 bq=google_big_query_global,
                                 authed_users_table_id=monitoring_bot_authed_users_table_id),
    telemetry_table_id=telemetry_sensors_table_id,
    sensor_id_ambient=sensor_id_ambient,
    sensor_id_bottom_tube=sensor_id_bottom_tube,
    error_string_id=error_string_id,
    ambient_alert_temperature=ambient_alert_temperature,
    bottom_tube_alert_temperature=bottom_tube_alert_temperature,
    tz=tz)

telemetry_processor: tp.TelemetryProcessor = tp.TelemetryProcessor(bq=google_big_query_global,
                                                                   alerting_bot=alerting_telegram_bot,
                                                                   location=location,
                                                                   telemetry_sensors_table_id=telemetry_sensors_table_id,
                                                                   sensor_id_bottom_tube=sensor_id_bottom_tube,
                                                                   sensor_id_ambient=sensor_id_ambient,
                                                                   error_string_id=error_string_id)


def on_new_telemetry(data, context) -> None:
    """
    https://cloud.google.com/functions/docs/writing/background
    """
    # print (context)
    # print (data)
    telemetry_processor.feed(data, context.event_id)


# noinspection PyShadowingNames
def on_telegram_alerting_bot_request(request: request):
    # print('Got request:{}'.format(request))
    # print('Data:\n{}'.format(request.data))
    alerting_telegram_bot.handle_request(request.get_json())


# noinspection PyShadowingNames
def on_telegram_monitoring_bot_request(request: request):
    monitoring_telegram_bot.handle_request(request.get_json())


def on_telegram_monitoring_bot_cron_request(request: request):
    # print('Got request:{}'.format(request))
    # print('Data:\n{}'.format(request.data))
    # Got request:<Request 'http://europe-west1-tarasovka-monitoring.cloudfunctions.net' [POST]>
    # Data:
    # b''
    monitoring_telegram_bot.compose_and_send_digest_to_all()
    from flask import Response
    return Response(status=200)

