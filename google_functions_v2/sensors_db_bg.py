from google.cloud import bigquery
import sensor_db as sdb
import datetime as dt
import typing as t
import sensor as sen
from collections import defaultdict
import bigquerydb as bigdb
import devicedatum as dd
from logger import logger

class SensorsDBBQ(sdb.SensorsDB):
    """
    This is a class that is responsible for reading/writing DevideDatum objects (our primary telemetry data) from/to a DB.
    """
    COL_TIMESTAMP_NAME: str = "Timestamp"
    COL_ERROR_MSG_NAME: str = "ErrorMessage"
    COL_TUBENAME_NAME: str = "TubeName"
    COL_TEMPERATURE_NAME: str = "Temperature"
    COL_TUBES_NAME: str = "Tubes"
    TABLE_NAME: str = "sensors_data_table"
    TIME_FORMAT: str = "%Y-%m-%d %H:%M:%S.%f"

    def __init__(self, db: bigdb.BigQueryDB):
        self.db = db

        def set_up_partitioning(tbl) -> bigquery.Table:
            tbl.clustering_fields = [self.COL_TIMESTAMP_NAME]

            tbl.time_partitioning = bigquery.TimePartitioning(
                type_=bigquery.TimePartitioningType.MONTH,
                field=self.COL_TIMESTAMP_NAME,
                expiration_ms=int((365 * 3 + 365 / 2) * 24 * 60 * 60 * 1000))
            return tbl

        self.table = self.db.create_table(table_name=self.TABLE_NAME,
                                     schema=[bigquery.SchemaField(self.COL_TIMESTAMP_NAME, "TIMESTAMP", mode="REQUIRED")],
                                     fields=[bigquery.SchemaField(self.COL_ERROR_MSG_NAME, "STRING", mode="NULLABLE"),
                                             bigquery.SchemaField(self.COL_TUBES_NAME, "RECORD", mode="REPEATED",
                                                                  fields=[bigquery.SchemaField(self.COL_TUBENAME_NAME, "STRING", mode="REQUIRED"),
                                                                          bigquery.SchemaField(self.COL_TEMPERATURE_NAME, "FLOAT", mode="REQUIRED")])],
                                     modify_table_callback=set_up_partitioning)


    def write(self, datum: dd.DeviceDatum) -> None:
        logger.info(f'datum: {datum}')
        tubes: t.List[t.Dict[str, float]] = []
        for name, temp in datum.name_to_temp.items():
            tubes.append({self.COL_TUBENAME_NAME: name, self.COL_TEMPERATURE_NAME: temp})

        str_datetime: str = datum.time.strftime(self.TIME_FORMAT)
        columns: t.Dict[str, str] = {self.COL_TIMESTAMP_NAME: str_datetime, self.COL_TUBES_NAME: tubes}
        if datum.error_msg != "":
            columns[self.COL_ERROR_MSG_NAME] = datum.error_msg

        errors = self.db.client.insert_rows(self.table, [columns])
        logger.info(f'Written into {self.table.table_id}, result: {errors if errors else "Ok"}')


    def read_starting_from(self, date: dt.datetime) -> t.Tuple[t.List[sen.Sensor], t.Set[str]]:
        logger.info(f'date: {date}')
        str_datetime: str = date.strftime(self.TIME_FORMAT)
        query: str = f"SELECT {self.COL_TIMESTAMP_NAME}, {self.COL_ERROR_MSG_NAME}, {self.COL_TUBES_NAME} FROM {self.table} WHERE {self.COL_TIMESTAMP_NAME} >= TIMESTAMP('{str_datetime}') ORDER BY {self.COL_TIMESTAMP_NAME}"
        query_job: bigquery.job.QueryJob = self.db.client.query(query)
        name_to_sensor_temp: t.Dict[str, sen.Sensor] = defaultdict(lambda: sen.Sensor(temperatures=[], name="", timestamps=[]))
        messages: t.Set[str] = set()

        for row in query_job:
            for item in row[2]:
                name_to_sensor_temp[item[self.COL_TUBENAME_NAME]].name = item[self.COL_TUBENAME_NAME]
                name_to_sensor_temp[item[self.COL_TUBENAME_NAME]].timestamps.append(row[0])
                name_to_sensor_temp[item[self.COL_TUBENAME_NAME]].temperatures.append(item[self.COL_TEMPERATURE_NAME])
            if row[1]:
                messages.add(row[1])
        logger.info(f'Read {len(name_to_sensor_temp)} temperature measurements and {len(messages)} messages. '
                    f'Query result: {query_job.error_result if query_job.error_result else "Ok"}')
        return list(name_to_sensor_temp.values()), messages



    def delete_before(self, date: dt.datetime) -> None:
        logger.info(f'date: {date}')
        str_datetime: str = date.strftime(self.TIME_FORMAT)
        query = f"DELETE FROM {self.table}  WHERE {self.COL_TIMESTAMP_NAME} < TIMESTAMP('{str_datetime}')"
        query_job: bigquery.job.QueryJob = self.db.client.query(query)
        logger.info(f'Query result: {query_job.error_result if query_job.error_result else "Ok"}')



# if __name__ == "__main__":
#     db = SensorsDBBQ(project="tarasovka", dataset_id="tarasovka", location="europe-west2")
#     datum_1: dd.DeviceDatum = dd.DeviceDatum({"Ambient": 11.2, "BottomTube": 15.6}, dt.datetime(2022, 9, 29, 2, 0, tzinfo=pytz.UTC), "")
#     datum_2: dd.DeviceDatum = dd.DeviceDatum({"Ambient": 13.2, "BottomTube": 18.6}, dt.datetime(2022, 9, 29, 2, 2, tzinfo=pytz.UTC), "")
#     db.write(datum_1)
#     db.write(datum_2)
