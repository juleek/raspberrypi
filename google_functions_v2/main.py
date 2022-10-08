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


PROJECT: str = "tarasovka"
DATASET_ID: str = "tarasovka"
LOCATION: str = "europe-west2"
TOPIC_ID: str = "tarasovka_topic"


@functions_framework.cloud_event
def google_ingest(cloud_event):
    chat_id_db: chidb.ChatIdDB = chidb.ChatIdDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    consumer_db: sdb.Consumer = sdb.Consumer(sdbq.SensorsDBBQ(project=PROJECT,
                                                              dataset_id=DATASET_ID,
                                                              location=LOCATION,
                                                              table_name="sensors_db"))
    consumer_alert: alt.Alerting = alt.Alerting(name_min_tuples=[{botnotif.AMBIENT_TUBE: botnotif.ambient_alert_temperature,
                                                                  botnotif.BOTTOM_TUBE: botnotif.bottom_tube_alert_temperature}],
                                                sender=tel_s.TelegramSender(chat_id_db.read(alt.NAME), alt.ID))


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
    chat_id: int = tel_s.get_chat_id_from_update_msg(request.data.decode("utf-8"))
    db: chidb.ChatIdDB = chidb.ChatIdDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    db.ask_to_add(chat_id, tel_s.TelegramSender(chat_id, botnotif.ID), botnotif.NAME)
    return 'OK'

@functions_framework.http
def on_alerting_bot_message(request: flask.Request):
    chat_id: int = tel_s.get_chat_id_from_update_msg(request.data.decode("utf-8"))
    db: chidb.ChatIdDB = chidb.ChatIdDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION)
    db.ask_to_add(chat_id, tel_s.TelegramSender(chat_id, alt.ID), alt.NAME)

    return 'OK'



@functions_framework.http
def on_cron(request: flask.Request):
    botnotif.notify(sdbq.SensorsDBBQ(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION),
                    chidb.ChatIdDB(project=PROJECT, dataset_id=DATASET_ID, location=LOCATION))

    return 'OK'
