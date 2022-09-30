#!/usr/bin/python3

# import sensors_db_bg as sdbq
from google.cloud import bigquery
import secrets_bot as sec_bot
import bigquerydb as bigdb
import sender as sen
import typing as t


class ChatIdDB:
    COL_BOT_NAME: str = "Bot_name"
    COL_CHAT_ID: str = "Chat_ID"
    TABLE_NAME: str = "chat_id_db"

    def __init__(self, project, dataset_id, location):
        self.project = project
        self.dataset_id = dataset_id
        self.location = location
        self.bigq_db = bigdb.BigQueryDB(project=project, dataset_id=dataset_id, location=location)
        self.client = self.bigq_db.client
        self.table = self.create_chat_iddb()
        self.bot_id = sec_bot.notifier_bot_id


    def create_chat_iddb(self) -> bigquery.Table:
        return self.bigq_db.create_table(self.bigq_db.dataset,
                                          table_name=self.TABLE_NAME,
                                          fields=[bigquery.SchemaField(self.COL_BOT_NAME, "STRING", mode="REQUIRED"),
                                                  bigquery.SchemaField(self.COL_CHAT_ID, "INTEGER", mode="REQUIRED")])



    def ask_to_add(self, chat_id: int, sender: sen.Sender, bot_name: str):
        text: str = f'Authenticating has not been implemented yet, so insert your chat id into Google BigQuery manually by issuing: ' \
               f'MERGE INTO tarasovka.chat_id_db AS Dst \
               USING      (SELECT "{bot_name}" AS Bot_name, {chat_id} AS Chat_ID) AS Src \
               ON         Dst.Bot_name = Src.Bot_name \
               WHEN MATCHED THEN     UPDATE SET Chat_ID = Src.Chat_ID \
               WHEN NOT MATCHED THEN INSERT (Bot_name, Chat_ID) VALUES (Src.Bot_name, Src.Chat_ID);' \
               f'at https://console.cloud.google.com/bigquery'

        sender.send_text(text)


    def read(self, bot_name: str) -> t.Optional[int]:
        query = f"SELECT {self.COL_CHAT_ID} FROM {self.table} WHERE {self.COL_BOT_NAME} = '{bot_name}'"
        query_job = self.client.query(query)

        for row in query_job:
            return row[0]
        return None
