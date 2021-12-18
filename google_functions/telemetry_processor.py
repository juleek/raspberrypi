from pathlib import Path
import sys
# import metrics
import datetime
import json
import base64

import big_query as bq
from google.cloud import bigquery
import bots

project_id: str = "tarasovka-monitoring"
metric_type_name: str = "telemetry_sensors/temperature"


# noinspection PyShadowingNames
class GBigQueryForSensors:
    def __init__(self,
                 bq: bq.GBigQuery,
                 table_id: str,
                 sensor_id_ambient: str,
                 sensor_id_bottom_tube: str,
                 error_string_id: str) -> None:
        self.bq = bq
        self.table_id = table_id
        self.sensor_id_ambient = sensor_id_ambient
        self.sensor_id_bottom_tube = sensor_id_bottom_tube
        self.error_string_id = error_string_id

        if self.bq.dry_run:
            return

        self.table = self.bq.create_table_if_not_created(self.table_id)

        schema = [
            bigquery.SchemaField('ContextEventId', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('Timestamp', 'TIMESTAMP', mode='REQUIRED'),
            bigquery.SchemaField(self.sensor_id_ambient, 'FLOAT64', mode='NULLABLE'),
            bigquery.SchemaField(self.sensor_id_bottom_tube, 'FLOAT64', mode='NULLABLE'),
            bigquery.SchemaField(self.error_string_id, 'STRING', mode='NULLABLE'),
        ]
        self.table = self.bq.ensure_table_scheme(existing_table=self.table, schema=schema)

    def insert_new_row(self, event_id: str, ambient_temperature, bottom_tube_temperature, error_string: str) -> None:
        timestamp = datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S.%f')
        rows_to_insert = [(event_id, timestamp, ambient_temperature, bottom_tube_temperature, error_string)]
        self.bq.insert_rows(table=self.table, rows_to_insert=rows_to_insert)


# noinspection PyShadowingNames
class TelemetryProcessor:
    def __init__(self,
                 bq: bq.GBigQuery,
                 alerting_bot: bots.AlertingTelegramBot,
                 location: str,
                 telemetry_sensors_table_id: str,
                 sensor_id_bottom_tube: str,
                 sensor_id_ambient: str,
                 error_string_id: str) -> None:
        # self.metrics = metrics.GMetrics(project_id=project_id,
                                        # metric_type_name=metric_type_name,
                                        # location=location,
                                        # namespace="global namespace")

        self.bq = GBigQueryForSensors(bq=bq,
                                      table_id=telemetry_sensors_table_id,
                                      sensor_id_bottom_tube=sensor_id_bottom_tube,
                                      sensor_id_ambient=sensor_id_ambient,
                                      error_string_id=error_string_id)
        self.alerting_bot = alerting_bot

    def feed(self, data, event_id) -> None:
        if 'data' not in data:
            print('There is no "data" key in "data", available keys are: {}'.format(data.keys()))  # Log Error
            return  # TODO: Log error

        json_data = base64.b64decode(data['data']).decode('utf-8')
        try:
            parsed_json = json.loads(json_data)
        except json.JSONDecodeError as exc:
            print('Failed to decode JSON: {}: {}'.format(type(exc), exc))  # Log Error
            return

        all_sensors = [self.bq.sensor_id_ambient, self.bq.sensor_id_bottom_tube]
        if not any(True for sensor in all_sensors if sensor in parsed_json):
            print('Found neither of "{}" in JSON. Available keys are: "{}"'.
                  format(all_sensors, parsed_json.keys()))  # Log Error
            return

        ambient_temperature = parsed_json[self.bq.sensor_id_ambient] \
            if self.bq.sensor_id_ambient in parsed_json else None
        bottom_tube_temperature = parsed_json[self.bq.sensor_id_bottom_tube] \
            if self.bq.sensor_id_bottom_tube in parsed_json else None
        error_string = parsed_json[self.bq.error_string_id] \
            if self.bq.error_string_id in parsed_json else None
        print("Checks are passed: bottom_tube_temperature={}, ambient_temperature={}, ErrorString={}".
              format(bottom_tube_temperature, ambient_temperature, error_string))

        self.bq.insert_new_row(event_id=event_id,
                               ambient_temperature=ambient_temperature,
                               bottom_tube_temperature=bottom_tube_temperature,
                               error_string=error_string)
        #self.metrics.add_time_series(self.bq.sensor_id_ambient, ambient_temperature)
        #self.metrics.add_time_series(self.bq.sensor_id_bottom_tube, bottom_tube_temperature)

        self.alerting_bot.alert_all_if_needed(ambient_temperature=ambient_temperature,
                                              bottom_tube_temperature=bottom_tube_temperature)


if __name__ == "__main__":
    print("asdf")
