#!/usr/bin/python3

import sensor_db
import telegram_sender as tel_s
import sensor as sen
import typing as t
import datetime as dt
import plot as pl
import chat_id_db as chidb
import secrets_bot
import sender
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


def format_time_delta(time: dt.timedelta, digits: int) -> str:
    seconds: float = time.total_seconds()
    isec, fsec = divmod(round(seconds * 10 ** digits), 10 ** digits)
    return f'{dt.timedelta(seconds=isec)}.{fsec:0{digits}.0f}'

def create_msg_with_current_temp(sensors: t.List[sen.Sensor], error_msgs: t.Set[str]) -> str:
    time_to_temp: col.OrderedDict[dt.datetime, t.List[t.Tuple[str, float]]] = col.OrderedDict()
    for sensor in sensors:
        assert(len(sensor.timestamps) == 1)
        assert(len(sensor.temperatures) == 1)
        time_to_temp.setdefault(sensor.timestamps[0], []).append((sensor.name, sensor.temperatures[0]))

    lines = []
    now = dt.datetime.now(pytz.utc)
    for time, names_and_temps in time_to_temp.items():
        time_ago: str = format_time_delta(now - time, 1)
        group: str = f'*{time_ago} ago \\({time.strftime("%H:%M:%S %d.%m")}\\)*:\n'
        for name, temperature in names_and_temps:
            group += f'\\* {name}: {temperature}°C\n'
        lines.append(group)

    if error_msgs:
        lines.append(f'\n*There are {len(error_msgs)} error messages*:')
        for error in error_msgs:
            lines.append(f'* {error}')
    return "\n".join(lines)


def send_current_temperature(sensorsdbbq: sensor_db.SensorsDB, sender: sender.Sender):
    sensors, error_msgs = sensorsdbbq.read_last_result()
    text: str = create_msg_with_current_temp(sensors, error_msgs)
    sender.send_text(text, is_markdown=True)



def dispatch_command(jsn, chat_id: int, chat_id_db: chidb.ChatIdDB, sensors_db: sensor_db.SensorsDB, sender: sender.Sender) -> None:
    if chat_id_db.exists(chat_id) == False:
        chat_id_db.ask_to_add(chat_id, sender, BOT_NAME)
        return

    # Telegram Bot API responses are represented as JSON-objects. https://core.telegram.org/bots/api#message
    text: str = jsn['message']['text']
    # 'entities' is Array of MessageEntity. https://core.telegram.org/bots/api#messageentity
    command_type: str = jsn['message']['entities'][0]["type"]
    if command_type == "bot_command" and text == "/gettemp":
        send_current_temperature(sensors_db, sender)

