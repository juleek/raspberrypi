#!/usr/bin/python3

import sensor_db
import telegram_sender as tel_s
import sensor as sen
import typing as t
import datetime as dt
import plot as pl
import chat_id_db as chidb
import secrets_bot
import datetime
import sender as send
import pytz
import collections as col

BOT_NAME: str = "notifier_bot"
BOT_SECRET: str = secrets_bot.notifier_bot_id
BOTTOM_TUBE_NAME: str = "BottomTube"
AMBIENT_TUBE_NAME: str = "Ambient"
AMBIENT_TEMP_THRESHOLD: float = 6.
BOTTOM_TEMP_THRESHOLD: float = 12.


def notify(db: sensor_db.SensorsDB, chat_db: chidb.ChatIdDB):
    chat_id: int = chat_db.read(BOT_NAME)
    if chat_id is None:
        return

    sensors, error_msgs = db.read_for_period(dt.timedelta(hours=24))
    fig, axes, png_buf = pl.create_plot(sensors=sensors,
                                        bottom_tube_alert_temp=BOTTOM_TEMP_THRESHOLD,
                                        ambient_tube_alert_temp=AMBIENT_TEMP_THRESHOLD,
                                        bottom_tube=BOTTOM_TUBE_NAME,
                                        ambient_tube=AMBIENT_TUBE_NAME)
    msg: str = create_msg(sensors, error_msgs)

    sender = tel_s.TelegramSender(chat_id, BOT_SECRET)
    sender.send_with_pic(msg, png_buf)


def create_msg(sensors: t.List[sen.Sensor], error_msgs: t.Set[str]) -> str:
    lines: t.List[str] = []
    for sensor in sensors:
        lines.append(f'{sensor.name}: Min: {min(sensor.temperatures)}, Max: {max(sensor.temperatures)}')
    if error_msgs:
        lines.append(f'\n\nThere are {len(error_msgs)} error messages:')
        lines.extend(error_msgs)
    return "\n".join(lines)



def create_msg_with_current_temp(sensors: t.List[sen.Sensor], error_msgs: t.Set[str]) -> str:
    time_to_temp: col.OrderedDict[dt.datetime, t.List[t.Tuple[str, float]]] = col.OrderedDict()
    for sensor in sensors:
        assert(len(sensor.timestamps) == 1)
        assert(len(sensor.temperatures) == 1)
        time_to_temp.setdefault(sensor.timestamps[0], []).append((sensor.name, sensor.temperatures[0]))

    lines = []
    now = datetime.datetime.now(pytz.utc)
    for time, names_and_temps in time_to_temp.items():
        time_ago: dt.timedelta = now - time
        time_without_microseconds = str(time_ago).split(".")[0]
        group: str = f'*{time_without_microseconds} ago \\({time.strftime("%H:%M %d.%m.%Y")}\\)*:\n'
        for name, temperature in names_and_temps:
            group += f'\\* {name}: {temperature}°C\n'
        lines.append(group)

    if error_msgs:
        lines.append(f'\n*There are {len(error_msgs)} error messages*:')
        for error in error_msgs:
            lines.append(f'* {error}')
    return "\n".join(lines)


def send_current_temperature(sensorsdbbq: sensor_db.SensorsDB, sender: send.Sender):
    sensors, error_msgs = sensorsdbbq.read_last_result()
    text: str = create_msg_with_current_temp(sensors, error_msgs)
    text = text.replace(".", "\\.")
    text = text.replace("-", "\\-")
    text = text.replace("`", "\\`")
    print(f'text = {text}')
    sender.send_text(text, is_markdown=True)



def dispatch_command(jsn, chat_id: int, chat_id_db: chidb.ChatIdDB, sensors_db: sensor_db.SensorsDB, sender: send.Sender) -> None:
    if chat_id_db.exists(chat_id) == False:
        chat_id_db.ask_to_add(chat_id, sender, BOT_NAME)
        return

    # Telegram Bot API responses are represented as JSON-objects. https://core.telegram.org/bots/api#message
    text: str = jsn['message']['text']
    # 'entities' is Array of MessageEntity. https://core.telegram.org/bots/api#messageentity
    command_type: str = jsn['message']['entities'][0]["type"]
    if command_type == "bot_command" and text == "/gettemp":
        send_current_temperature(sensors_db, sender)



# if __name__ == "__main__":
#     create_msg([sen.Sensor(temperatures=[25.1, 25.1, 25.1, 28.2, 28.2, 25.1],
#                             name='example',
#                             timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)]),
#                  sen.Sensor(temperatures=[25.2, 26.1, 26.1],
#                             name='test',
#                             timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])],
#                 {'error', 'error2'})

# if __name__ == "__main__":
#     notify(db=sensor_db.SensorsDB(project="tarasovka", dataset_id="tarasovka", location="europe-west2"),
#            chat_db=chidb.ChatIdDB(project="tarasovka", dataset_id="tarasovka", location="europe-west2"))


# '"message":{"message_id":29,"from":{"id":1759739764,"is_bot":false,"first_name":"Di","last_name":"Di"},' \
# '"chat":{"id":-870776899,"title":"Alerting Tarasovka","type":"group","all_members_are_administrators":true},' \
# '"date":1669981560,"text":"/getinfo","entities":[{"offset":0,"length":8,"type":"bot_command"}]}}'


# import bigquerydb as bigdb
# import sensors_db_bg as sdbq
# if __name__ == "__main__":
#     bigquerydb: bigdb.BigQueryDB = bigdb.BigQueryDB(project="tarasovka", dataset_id="tarasovka", location="europe-west2")
#     send_current_temperature(sensorsdbbq=sdbq.SensorsDBBQ(bigquerydb), sender=tel_s.TelegramSender(-670407039, BOT_SECRET))

    # time_to_temp:  col.OrderedDict[dt.datetime, t.List[t.Tuple[str, float]]] = col.OrderedDict({dt.datetime(2022, 11, 4, 0, 0, tzinfo=dt.timezone.utc): [("test1", 14.2)],
    #                                                                                             dt.datetime(2022, 11, 5, 0, 0, tzinfo=dt.timezone.utc): [("test2", 6.2)]})
    # error_msgs = ["asdf.qwer 1234!@#$@%^$%^", "...-*/-*/-~~~"]
