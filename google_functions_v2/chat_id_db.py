#!/usr/bin/python3

# import sensors_db_bg as sdbq
from google.cloud import bigquery
import bigquerydb as bigdb
import sender as sen
import typing as t

class ChatIdDB:
    COL_BOT_NAME: str = "bot_name"
    COL_CHAT_ID: str = "chat_id"
    TABLE_NAME: str = "chat_ids"

    def __init__(self, project: str, dataset_id: str, location: str):
        self.db = bigdb.BigQueryDB(project=project, dataset_id=dataset_id, location=location)
        self.table = self.create_chat_ids_table()


    def create_chat_ids_table(self) -> bigquery.Table:
        return self.db.create_table(
                                          table_name=self.TABLE_NAME,
                                          fields=[bigquery.SchemaField(self.COL_BOT_NAME, "STRING", mode="REQUIRED"),
                                                  bigquery.SchemaField(self.COL_CHAT_ID, "INTEGER", mode="REQUIRED")])


    def ask_to_add(self, chat_id: int, sender: sen.Sender, bot_name: str):
        text: str = f'Authenticating has not been implemented yet, so insert your chat id into Google BigQuery manually by issuing: \
                    ```SQL\
                    MERGE INTO {self.table} AS Dst \
                    USING (SELECT "{bot_name}" AS Bot_name, {chat_id} AS Chat_ID) AS Src \
                    ON Dst.Bot_name = Src.Bot_name \
                    WHEN MATCHED THEN UPDATE SET Chat_ID = Src.Chat_ID \
                    WHEN NOT MATCHED THEN INSERT (Bot_name, Chat_ID) VALUES (Src.Bot_name, Src.Chat_ID); \
                    ```\
                    at https://console.cloud.google.com/bigquery'


        sender.send_text(text,is_markdown=False)


    def read(self, bot_name: str) -> t.Optional[int]:
        query: str = f"SELECT {self.COL_CHAT_ID} FROM {self.table} WHERE {self.COL_BOT_NAME} = '{bot_name}'"
        query_job: bigquery.job.QueryJob = self.db.client.query(query)

        for row in query_job:
            return row[0]
        return None
