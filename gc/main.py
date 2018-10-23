import sys
from google.cloud import bigquery
import google.api_core.exceptions


class GCloudState:
   def __init__(self, dataset_id, table_id, location):
      self.dataset_id  = dataset_id
      self.table_id    = table_id
      self.location    = location
      self.client      = bigquery.Client();

      # https://cloud.google.com/functions/docs/bestpractices/tips#functions-graceful-termination-python
      # https://googleapis.github.io/google-cloud-python/latest/bigquery/usage/datasets.html
      self.dataset_ref = self.client.dataset(self.dataset_id)
      datasets = list(self.client.list_datasets())
      # print (datasets)
      has_required_dataset = bool(datasets and [True for dataset in datasets if dataset.dataset_id == self.dataset_id])
      # print (has_required_dataset);
      # print('Datasets in project "{}":'.format(project))
      if not has_required_dataset:
         print('Project "{}" does not contain dataset "{}" => creating it'.format(self.client.project, self.dataset_id))
         create_dataset(self)

      schema = [
          bigquery.SchemaField('ContextEventId'     , 'STRING'   , mode='REQUIRED'),
          bigquery.SchemaField('Timestamp'          , 'TIMESTAMP', mode='REQUIRED'),
          bigquery.SchemaField(sensor_id_ambient    , 'FLOAT64'  , mode='NULLABLE'),
          bigquery.SchemaField(sensor_id_bottom_tube, 'FLOAT64'  , mode='NULLABLE'),
          bigquery.SchemaField(error_string_id      , 'STRING'   , mode='NULLABLE'),
      ]
      self.table_ref = self.dataset_ref.table(self.table_id)
      self.table     = bigquery.Table(self.table_ref, schema=schema)
      tables = list(self.client.list_tables(self.dataset_ref))  # API request(s)
      # print (tables)
      has_required_table = bool(tables and [True for table in tables if table.table_id == self.table_id])
      if not has_required_table:
         print('Dataset "{}" in project "{}" does not contain table "{}" => creating it'.
                format(self.dataset_id, self.client.project, self.table_id))
         create_table(self)


def create_dataset(gcloud_state):
   dataset = bigquery.Dataset(gcloud_state.dataset_ref)
   dataset.location = location
   try:
      dataset = gcloud_state.client.create_dataset(dataset)  # API request
      print('Dataset "{}" created.'.format(dataset.dataset_id))
   except google.api_core.exceptions.AlreadyExists as exc:
      print('{}: {}'.format(type(exc).__name__, exc))
   except google.api_core.exceptions.Conflict as exc:
      print('{}: {}'.format(type(exc), exc))


def create_table(gcloud_state):
   try:
      gcloud_state.table = gcloud_state.client.create_table(gcloud_state.table)  # API request
      print("Table schema: {}".format(table.schema))
      print("Table num of rows: {}".format(table.num_rows))
   except google.api_core.exceptions.Conflict as exc:
      print('{}: {}'.format(type(exc), exc))


# https://cloud.google.com/bigquery/streaming-data-into-bigquery#bigquery-stream-data-python
def insert_new_row(gcloud_state, ambient_temperature, bottom_tube_temperature, error_string):
   rows_to_insert = [(u'evid2', u'2018-02-23 14:56', ambient_temperature, bottom_tube_temperature, error_string)]
   print ('Inserting: {}'.format(rows_to_insert))
   errors = gcloud_state.client.insert_rows(gcloud_state.table, rows_to_insert)  # API request
   print (errors)
   assert errors == []



# -------------------------------------------------------------------------------------------------------------------



sensor_id_bottom_tube   = "BottomTube";
sensor_id_ambient       = "Ambient";
error_string_id         = "ErrorString"
gcloud_state_global_var = GCloudState(dataset_id = "MainDataSet",
                                      table_id   = "AllTempSensors",
                                      location   = 'europe-west2');


def on_new_telemetry(data, context): # https://cloud.google.com/functions/docs/writing/background
   import base64
   import json

   if 'data' not in data:
      print('There is no "data" key in "data", available keys are: {}'.format(data.keys())); # Log Error
      return # TODO: Log error

   json_data = base64.b64decode(data['data']).decode('utf-8')
   try:
      json = json.loads(json_data)
   except JSONDecodeError as exc:
      print('Failed to decode JSON: {}: {}'.format(type(exc), exc)); # Log Error
      return;

   all_sensors = [sensor_id_ambient, sensor_id_bottom_tube]
   if not any(True for sensor in all_sensors if sensor in json):
      print('Found neither of "{}" in JSON. Available keys are: "{}"'.format(all_sensors, json.keys())); # Log Error
      return;

   ambient_temperature     = json[sensor_id_ambient]     if sensor_id_ambient     in json else None
   bottom_tube_temperature = json[sensor_id_bottom_tube] if sensor_id_bottom_tube in json else None
   error_string            = json[error_string_id]       if error_string_id       in json else None
   print ("Checks are passed: bottom_tube_temperature={}, ambient_temperature={}, ErrorString={}".
         format(bottom_tube_temperature, ambient_temperature, error_string))
   insert_new_row(gcloud_state            = gcloud_state_global_var,
                  ambient_temperature     = ambient_temperature,
                  bottom_tube_temperature = bottom_tube_temperature,
                  error_string            = error_string)



import base64
x = base64.b64encode(b'{ "BottomTube":12.5, "age":30, "city":"New York"}')
on_new_telemetry({'data': x}, None)
