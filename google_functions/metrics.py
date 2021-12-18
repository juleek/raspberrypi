from pathlib import Path
import sys
import time
from google.cloud import monitoring_v3


class GMetrics:
    def __init__(self, project_id: str, metric_type_name: str, location: str, namespace: str) -> None:
        self.client = monitoring_v3.MetricServiceClient()
        self.project_id = project_id
        self.metric_type_name = 'custom.googleapis.com/' + metric_type_name
        self.location = location
        self.namespace = namespace

        self.place_label_id = "place"
        self.project_name = self.client.project_path(self.project_id)
        self.__create_descriptor()

    def __create_descriptor(self) -> None:
        # https: // github.com / googleapis / googleapis / blob / master / google / api / metric.proto
        descriptor = monitoring_v3.types.MetricDescriptor()
        descriptor.type = self.metric_type_name
        descriptor.metric_kind = monitoring_v3.enums.MetricDescriptor.MetricKind.GAUGE
        descriptor.value_type = monitoring_v3.enums.MetricDescriptor.ValueType.DOUBLE
        descriptor.description = 'Temperature readings.'

        # https: // github.com / googleapis / googleapis / blob / master / google / api / label.proto
        place_label = monitoring_v3.types.LabelDescriptor()
        place_label.description = "Place of sensor"
        place_label.key = self.place_label_id
        descriptor.labels.extend([place_label])

        # print('Metrics descriptor "{}" has been created'.format(descriptor))
        # Metrics descriptor
        # "labels {
        #     key: "place"
        #     description: "Place of sensor"
        # }
        # metric_kind: GAUGE
        # value_type: DOUBLE
        # description: "Temperature readings."
        # type: "custom.googleapis.com/telemetry_sensors/temperature"
        # " has been created
        #

        # noinspection PyUnusedLocal
        descriptor = self.client.create_metric_descriptor(self.project_name, descriptor)
        # print('Metrics descriptor "{}" has been created'.format(descriptor))

    def delete_descriptor(self) -> None:
        descriptor_name = 'projects/{}/metricDescriptors/{}'.format(self.project_id, self.metric_type_name)
        self.client.delete_metric_descriptor(descriptor_name)
        print('Deleted metric descriptor "{}"'.format(descriptor_name))

    def add_time_series(self, place: str, temperature: float) -> None:
        if not temperature:
            return

        series = monitoring_v3.types.TimeSeries()

        # https://github.com/googleapis/googleapis/blob/master/google/monitoring/v3/metric.proto
        series.metric.type = self.metric_type_name
        series.metric.labels[self.place_label_id] = place

        # https://cloud.google.com/monitoring/api/resources#tag_generic_task
        series.resource.type = "generic_task"
        series.resource.labels["location"] = self.location
        series.resource.labels['namespace'] = self.namespace
        series.resource.labels["job"] = "temperature collecting job"
        series.resource.labels["task_id"] = "some task id"
        # series.resource.labels["place"] = "bottom_tube"

        point = series.points.add()
        point.value.double_value = temperature
        now = time.time()
        point.interval.end_time.seconds = int(now)
        point.interval.end_time.nanos = int((now - point.interval.end_time.seconds) * 10 ** 9)

        self.client.create_time_series(self.project_name, [series])
        # print('Successfully wrote time series: {}'.format(series))
        # Successfully wrote time series:
        # metric {
        #   labels { key: "place" value: "BottomTube"  }
        #   type: "custom.googleapis.com/telemetry_sensors/temperature"
        # }
        # resource {
        #   type: "generic_task"
        #   labels { key: "job" value: "temperature collecting job" }
        #   labels { key: "location" value: "europe-west2" }
        #   labels { key: "namespace" value: "global namespace" }
        #   labels { key: "task_id" value: "some task id" }
        # }
        # points {
        #   interval { end_time { seconds: 1541423655 nanos: 276073217 } }
        #   value { double_value: 12.0 }
        # }


if __name__ == "__main__":
    metrics = GMetrics("tarasovka-monitoring",
                       "telemetry_sensors/temperature",
                       "europe-west2",
                       namespace="test namespace")
    metrics.delete_descriptor()
    # metrics.add_time_series("BottomTube", 14)
    # metrics.add_time_series("Ambient", None)
