import base64
from json import JSONDecodeError
import big_query as bq


project_id: str = "tarasovka-monitoring"
location="europe-west2"
metric_type_name: str = "telemetry_sensors/temperature"

sensor_id_bottom_tube: str = "BottomTube"
sensor_id_ambient: str = "Ambient"
error_string_id: str = "ErrorString"

google_big_query_global: bq.GBigQuery = bq.GBigQuery(dataset_id="MainDataSet",
                                                     table_id="AllTempSensors",
                                                     location=location,
                                                     sensor_id_bottom_tube=sensor_id_bottom_tube,
                                                     sensor_id_ambient=sensor_id_ambient,
                                                     error_string_id=error_string_id,
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
    google_big_query_global.insert_new_row(event_id=event_id,
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
