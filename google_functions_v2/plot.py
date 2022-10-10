#!/usr/bin/python3

from sensor import Sensor
import typing as t
import matplotlib.pyplot as mplplt
import matplotlib.dates as mpldates
from pandas.plotting import register_matplotlib_converters
import numpy as np
import datetime as dt
import sensor as sen
from dataclasses import dataclass, field
import pytz
import io

@dataclass
class PlotLine:
    legend: str
    colour: t.Tuple[float, float, float]
    x: t.List[dt.datetime] = field(default_factory=list)
    y: np.ndarray = np.empty(0)
    threshold_hline: float = None


@dataclass
class PlotInfo:
    title: str
    tz: pytz.timezone = pytz.timezone("Europe/Moscow")
    lines: t.List[PlotLine] = field(default_factory=list)
    dpi: int = 500
    title_font_size: int = 23
    legend_font_size: int = 19
    format: str = 'png'


def make_plot(plot_info: PlotInfo):
    register_matplotlib_converters()
    fig, axes = mplplt.subplots()
    fig.suptitle(plot_info.title, fontsize=plot_info.title_font_size)

    for line in plot_info.lines:
        x: np.ndarray = np.empty(0)
        for qw in line.x:
            print(f'qw = {qw}')
            x = np.append(x, mpldates.date2num(qw))
        print(f'x = {x}')
        y: np.ndarray = np.sin(x) * 15 + 20 if not line.y.size else line.y
        print(f'y = {y}')
        axes.plot_date(x,
                       y,
                       linestyle='-',
                       color=line.colour,
                       label=line.legend,
                       markersize=3,
                       tz=plot_info.tz  # tz=timezone(timedelta(hours=3))
                       )
        if line.threshold_hline is not None:
            axes.axhline(line.threshold_hline, color=line.colour, linestyle='-.')
    axes.legend(loc='best', fontsize=plot_info.legend_font_size)
    axes.tick_params(labelright=True, labelsize=17)
    mplplt.xticks(rotation=40)
    mplplt.grid(True)

    png_buf: io.BytesIO = io.BytesIO()
    fig.savefig(png_buf, format=plot_info.format, dpi=plot_info.dpi, bbox_inches='tight')
    png_buf.seek(0)

    return fig, axes, png_buf



def create_plot(sensors: t.List[sen.Sensor],
                bottom_tube_alert_temp: float,
                ambient_tube_alert_temp: float,
                bottom_tube: str,
                ambient_tube: str) -> t.Tuple[t.Any, t.Any, io.BytesIO]:
    bottom_tube_line: PlotLine = PlotLine(legend="BottomTube",
                                          colour=(1, 0, 0),
                                          threshold_hline=bottom_tube_alert_temp)
    ambient_line: PlotLine = PlotLine(legend="Ambient",
                                      colour=(0, 0, 1),
                                      threshold_hline=ambient_tube_alert_temp)


    for sensor in sensors:
        if sensor.name == bottom_tube:
            bottom_tube_line.x = sensor.timestamps
            bottom_tube_line.y = np.append(bottom_tube_line.y, sensor.temperatures)

        if sensor.name == ambient_tube:
            ambient_line.x = sensor.timestamps
            ambient_line.y = np.append(ambient_line.y, sensor.temperatures)

    plot_info: PlotInfo = PlotInfo(title='Temp in Tarasovka on {}'.
                                   format(dt.datetime.now(PlotInfo.tz).strftime('%d.%m  %H:%M')),
                                   tz=PlotInfo.tz)

    plot_info.lines.append(bottom_tube_line)
    plot_info.lines.append(ambient_line)
    fig, axes, png_buf = make_plot(plot_info)
    return fig, axes, png_buf


