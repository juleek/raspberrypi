import telemetry_processor as tp
import big_query as bq
from flask import request
import bots
import secrets

location: str = "europe-west2"
dataset_id: str = "MainDataSet"
alerting_bot_authed_users_table_id: str = "AlertingBotChats"
monitoring_bot_authed_users_table_id: str = "MonitoringBotChats"

google_big_query_global: bq.GBigQuery = bq.GBigQuery(dataset_id=dataset_id,
                                                     location=location,
                                                     dry_run=False)

alerting_telegram_bot: bots.AlertingTelegramBot = bots.AlertingTelegramBot(
    bot=bots.BigQueryTelegramBot(bot=bots.TelegramBot(secrets.alerting_telegram_bot_token),
                                 bq=google_big_query_global,
                                 authed_users_table_id=alerting_bot_authed_users_table_id),
    ambient_temp_threshold=6,
    bottom_tube_temp_threshold=12
)

telemetry_processor: tp.TelemetryProcessor = tp.TelemetryProcessor(bq=google_big_query_global,
                                                                   alerting_bot=alerting_telegram_bot,
                                                                   location=location)


# bq_monitoring_telegram_bot: bots.BigQueryTelegramBot = bots.BigQueryTelegramBot(
#     bot=bots.TelegramBot(monitoring_telegram_bot_token),
#     bq=google_big_query_global,
#     authed_users_table_id=monitoring_bot_authed_users_table_id)


def on_new_telemetry(data, context) -> None:
    """
    https://cloud.google.com/functions/docs/writing/background
    """
    # print (context)
    telemetry_processor.feed(data, context.event_id)


# noinspection PyShadowingNames
def on_telegram_alerting_bot_request(request: request):
    alerting_telegram_bot.handle_request(request.get_json())

# def on_telegram_monitoring_bot_request(request: request):
#     bq_monitoring_telegram_bot.handle_request(request.get_json())
