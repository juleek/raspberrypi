#!/usr/bin/python3

import sensors_db_bg as sdbq
import telegram_sender as tel_s
import sensor as sen
import typing as t
import datetime as dt
import plot as pl
import chat_id_db as chidb
import secrets_bot

BOT_NAME: str = "notifier_bot"
BOT_SECRET: str = secrets_bot.notifier_bot_id
BOTTOM_TUBE_NAME: str = "BottomTube"
AMBIENT_TUBE_NAME: str = "Ambient"
AMBIENT_TEMP_THRESHOLD: float = 6.
BOTTOM_TEMP_THRESHOLD: float = 12.


def notify(db: sdbq.SensorsDBBQ, chat_db: chidb.ChatIdDB):
    chat_id: int = chat_db.read(BOT_NAME)
    if chat_id is None:
        return

    sensors, error_msgs = db.read_for_period(dt.timedelta(hours=48))
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


# if __name__ == "__main__":
#     create_msg([sen.Sensor(temperatures=[25.1, 25.1, 25.1, 28.2, 28.2, 25.1],
#                             name='example',
#                             timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)]),
#                  sen.Sensor(temperatures=[25.2, 26.1, 26.1],
#                             name='test',
#                             timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])],
#                 {'error', 'error2'})

# if __name__ == "__main__":
#     notify(db=sdbq.SensorsDBBQ(project="tarasovka", dataset_id="tarasovka", location="europe-west2"),
#            chat_db=chidb.ChatIdDB(project="tarasovka", dataset_id="tarasovka", location="europe-west2"))
