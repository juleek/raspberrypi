#!/usr/bin/python3

import sensors_db_bg as sdbq
import telegram_sender as tel_s
import sensor as sen
import typing as t
import datetime as dt
import plot as pl
import chat_id_db as chidb
import secrets_bot as sec_bot

NAME: str = "Notifier_bot"
ID: str = sec_bot.notifier_bot_id
BOTTOM_TUBE: str = "BottomTube"
AMBIENT_TUBE: str = "Ambient"
ambient_alert_temperature: float = 6
bottom_tube_alert_temperature: float = 12


def notify(db: sdbq.SensorsDBBQ, chat_db: chidb.ChatIdDB):
    chat_id: int = chat_db.read(NAME)
    if chat_id is None:
        return

    sensors, error_msgs = db.read_for_period(dt.timedelta(hours=48))
    fig, axes, png_buf = pl.create_plot(sensors,
                                        bottom_tube_alert_temperature,
                                        ambient_alert_temperature,
                                        BOTTOM_TUBE,
                                        AMBIENT_TUBE)
    msg: str = create_msg(sensors, error_msgs)

    sender = tel_s.TelegramSender(chat_id, ID)
    sender.send_with_pic(msg, png_buf)


def create_msg(sensors: t.List[sen.Sensor], error_msgs: t.Set[str]) -> str:
    lines: t.List[str] = []
    for sensor in sensors:
        lines.append(f'{sensor.name}: Min: {min(sensor.temperatures)}, Max: {max(sensor.temperatures)}')
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
