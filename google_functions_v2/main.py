#!/usr/bin/python3

import functions_framework
import ingest as ing
import sensor_db as sdb
import bot_alerting as alt
import telegram_sender as tel_s
import flask
import devicedatum as dd
import sensors_db_bg as sdbq
import topic as tp
import base64
import chat_id_db as chidb
import bot_notifier as botnotif
import bigquerydb as bigdb


PROJECT: str = "tarasovka"
DATASET_ID: str = "tarasovka"
LOCATION: str = "europe-west2"
TOPIC_ID: str = "tarasovka_topic"


@functions_framework.cloud_event
def google_ingest(cloud_event):
    bigquerydb = bigdb.BigQueryDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    chat_id_db: chidb.ChatIdDB = chidb.ChatIdDB(db=bigquerydb)
    consumer_db: sdb.Consumer = sdb.Consumer(sdbq.SensorsDBBQ(db=bigquerydb))
    consumer_alert: alt.Alerting = alt.Alerting(name_to_min={botnotif.AMBIENT_TUBE_NAME: botnotif.AMBIENT_TEMP_THRESHOLD,
                                                             botnotif.BOTTOM_TUBE_NAME: botnotif.BOTTOM_TEMP_THRESHOLD},
                                                sender=tel_s.TelegramSender(chat_id_db.read(alt.BOT_NAME), alt.BOT_ID))

    ingest = ing.Ingest([consumer_db, consumer_alert])

    msg: str = base64.b64decode(cloud_event.data['message']['data']).decode("utf-8")
    ingest.onDatum(dd.DeviceDatum.from_json(msg))

    return 'OK'


@functions_framework.http
def google_write_msg_to_topic(request: flask.Request):
    tp.create_topic_and_publish_msg(PROJECT,
                                    TOPIC_ID,
                                    request.data.decode("utf-8"))

    return 'OK'


@functions_framework.http
def on_notifier_bot_message(request: flask.Request):
    bigquerydb = bigdb.BigQueryDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    chat_id: int = tel_s.get_chat_id_from_update_msg(request.data.decode("utf-8"))
    db: chidb.ChatIdDB = chidb.ChatIdDB(db=bigquerydb)
    db.ask_to_add(chat_id, tel_s.TelegramSender(chat_id, botnotif.BOT_SECRET), botnotif.BOT_NAME)
    return 'OK'


@functions_framework.http
def on_alerting_bot_message(request: flask.Request):
    bigquerydb = bigdb.BigQueryDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    chat_id: int = tel_s.get_chat_id_from_update_msg(request.data.decode("utf-8"))
    db: chidb.ChatIdDB = chidb.ChatIdDB(db=bigquerydb)
    db.ask_to_add(chat_id, tel_s.TelegramSender(chat_id, alt.BOT_ID), alt.BOT_NAME)

    return 'OK'


@functions_framework.http
def on_cron(request: flask.Request):
    bigquerydb = bigdb.BigQueryDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    botnotif.notify(sdbq.SensorsDBBQ(db=bigquerydb),
                    chidb.ChatIdDB(db=bigquerydb))

    return 'OK'
