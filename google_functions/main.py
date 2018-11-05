from json import JSONDecodeError
from google.cloud import bigquery
import google.api_core.exceptions
import base64
import datetime


class GCloudState:
    def __init__(self, dataset_id: str, table_id: str, location: str, dry_run: bool = False) -> None:
        self.dataset_id = dataset_id
        self.table_id = table_id
        self.location = location
        self.client = bigquery.Client()
        self.dry_run = dry_run

        if self.dry_run:
            return

        # https://cloud.google.com/functions/docs/bestpractices/tips#functions-graceful-termination-python
        # https://googleapis.github.io/google-cloud-python/latest/bigquery/usage/datasets.html

        # datasets = list(self.client.list_datasets())
        # has_required_dataset = bool(datasets and
        #                             [True for dataset in datasets if dataset.dataset_id == self.dataset_id])

        self.dataset_ref = self.client.dataset(self.dataset_id)
        try:
            self.client.get_dataset(self.dataset_ref)
        except google.api_core.exceptions.NotFound:
            print('Project "{}" does not contain dataset "{}" => creating it'.format(self.client.project,
                                                                                     self.dataset_id))
            self.__create_dataset()

        self.table_ref = self.dataset_ref.table(self.table_id)
        try:
            self.table = self.client.get_table(self.table_ref)
        except google.api_core.exceptions.NotFound:
            print('Dataset "{}" in project "{}" does not contain table "{}" => creating it'.
                  format(self.dataset_id, self.client.project, self.table_id))
            self.table = self.__create_table()

        self.__ensure_table_scheme()

    def __create_dataset(self) -> None:
        dataset = bigquery.Dataset(self.dataset_ref)
        dataset.location = self.location
        try:
            dataset = self.client.create_dataset(dataset)  # API request
            print('Dataset "{}" created.\n'.format(dataset.dataset_id))
        except google.api_core.exceptions.AlreadyExists as exc:
            print('{}: {}'.format(type(exc).__name__, exc))
        except google.api_core.exceptions.Conflict as exc:
            print('{}: {}'.format(type(exc), exc))

    def __create_table(self):
        try:
            self.client.create_table(bigquery.Table(self.table_ref))  # API request
            print('Table "{}" created.\n'.format(self.table_ref))
        except google.api_core.exceptions.Conflict as exc:
            print('{}: {}'.format(type(exc), exc))

        table = self.client.get_table(self.table_ref)
        return table

    def __ensure_table_scheme(self):
        schema = [
            bigquery.SchemaField('ContextEventId', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('Timestamp', 'TIMESTAMP', mode='REQUIRED'),
            bigquery.SchemaField(sensor_id_ambient, 'FLOAT64', mode='NULLABLE'),
            bigquery.SchemaField(sensor_id_bottom_tube, 'FLOAT64', mode='NULLABLE'),
            # bigquery.SchemaField(error_string_id, 'STRING', mode='NULLABLE'),
        ]
        original_schema = self.table.schema
        # print ('Original schema: {}'.format(original_schema))
        new_schema = original_schema[:]  # creates a copy of the schema
        for schema_field in schema:
            if not [True for existing_field in original_schema if existing_field.name == schema_field.name]:
                print('{} is not in original schema of table "{}" => adding it'.format(schema_field,
                                                                                       self.table.table_id))
                new_schema.append(schema_field)

        self.table.schema = new_schema
        self.table = self.client.update_table(self.table, ['schema'])  # API request

    def insert_new_row(self, event_id: str, ambient_temperature, bottom_tube_temperature, error_string: str) -> None:
        """
        https://cloud.google.com/bigquery/streaming-data-into-bigquery#bigquery-stream-data-python
        """

        if self.dry_run:
            return

        timestamp = datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S.%f')
        rows_to_insert = [(event_id, timestamp, ambient_temperature, bottom_tube_temperature
                           # , error_string
                           )]
        print('Inserting: {}'.format(rows_to_insert))
        errors = self.client.insert_rows(self.table, rows_to_insert)  # API request
        if errors:
            print(errors)
            assert errors == []


# =====================+=====================+=====================+=====================+=====================+====


sensor_id_bottom_tube: str = "BottomTube"
sensor_id_ambient: str = "Ambient"
error_string_id: str = "ErrorString"
gcloud_state_global_var: GCloudState = GCloudState(dataset_id="MainDataSet",
                                                   table_id="AllTempSensors",
                                                   location="europe-west2",
                                                   dry_run=True)


def on_new_telemetry_impl(data, event_id) -> None:
    import base64
    import json

    if 'data' not in data:
        print('There is no "data" key in "data", available keys are: {}'.format(data.keys()))  # Log Error
        return  # TODO: Log error

    json_data = base64.b64decode(data['data']).decode('utf-8')
    try:
        json = json.loads(json_data)
    except JSONDecodeError as exc:
        print('Failed to decode JSON: {}: {}'.format(type(exc), exc))  # Log Error
        return

    all_sensors = [sensor_id_ambient, sensor_id_bottom_tube]
    if not any(True for sensor in all_sensors if sensor in json):
        print('Found neither of "{}" in JSON. Available keys are: "{}"'.format(all_sensors, json.keys()))  # Log Error
        return

    ambient_temperature = json[sensor_id_ambient] if sensor_id_ambient in json else None
    bottom_tube_temperature = json[sensor_id_bottom_tube] if sensor_id_bottom_tube in json else None
    error_string = json[error_string_id] if error_string_id in json else None
    # print("Checks are passed: bottom_tube_temperature={}, ambient_temperature={}, ErrorString={}".
    #          format(bottom_tube_temperature, ambient_temperature, error_string))
    gcloud_state_global_var.insert_new_row(event_id=event_id,
                                           ambient_temperature=ambient_temperature,
                                           bottom_tube_temperature=bottom_tube_temperature,
                                           error_string=error_string)


def on_new_telemetry(data, context) -> None:
    """
    https://cloud.google.com/functions/docs/writing/background
    """
    # print (context)
    on_new_telemetry_impl(data, context.event_id)


x = base64.b64encode(b'{ "BottomTube":12.5, "Ambient":30}')
on_new_telemetry_impl({'data': x}, "TestContext")
