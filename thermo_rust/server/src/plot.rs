
type XY = (chrono::DateTime<chrono::Utc>, f64);

type Rgb = (u8, u8, u8);


pub struct Sensor {
   name: String,
   min: f64,
   curve: Vec<XY>,
   colour: Rgb,
}


fn format_date(x: &chrono::DateTime<chrono::Utc>) -> String { x.format("%m-%d %H").to_string() }

pub fn create_plot(sensors: &mut Vec<Sensor>) -> Result<(), Box<dyn std::error::Error>> {
   use plotters::drawing::IntoDrawingArea;
   let drawing_area = plotters::prelude::BitMapBackend::new("plot.png", (700, 700)).into_drawing_area();
   drawing_area.fill(&plotters::prelude::WHITE)?;

   for sensor in &mut *sensors {
      sensor.curve.sort_by_key(|elem| elem.0);
   }
   sensors.retain(|s| s.curve.is_empty() == false);

   if sensors.is_empty() {
      return Ok(());
   }

   let min_x = sensors.iter().map(|s| s.curve.first().unwrap().0).min().unwrap();
   let max_x = sensors.iter().map(|s| s.curve.last().unwrap().0).max().unwrap();
   println!("!!! min_x: {}, max_x: {}", min_x, max_x);

   let (min_y, max_y) = sensors
      .iter()
      .flat_map(|s| s.curve.iter().map(|p| p.1))
      .chain(sensors.iter().map(|s| s.min))
      .filter(|y| y.is_nan() == false)
      .fold((None, None), |(min, max), y| {
         let min = Some(min.unwrap_or(y).min(y));
         let max = Some(max.unwrap_or(y).max(y));
         (min, max)
      });
   let (min_y, max_y) = (min_y.unwrap(), max_y.unwrap());

   let current_time = chrono::Utc::now()
      .with_timezone(&chrono_tz::Europe::Moscow)
      .format("%d.%m  %H:%M")
      .to_string();
   let title = format!("Temp in Tarasovka on {}", current_time);


   let mut chart_builder = plotters::prelude::ChartBuilder::on(&drawing_area);
   chart_builder
      .margin(20)
      .x_label_area_size(40)
      .y_label_area_size(40)
      .caption(title, ("sans-serif", 40, &plotters::prelude::BLACK));
   let mut chart_context = chart_builder.build_cartesian_2d(min_x..max_x, min_y - 2.0..max_y + 2.0).unwrap();
   chart_context
      .configure_mesh()
      .x_label_formatter(&|x| {
        if x == &min_x {  // Check if this is the starting point
            String::new()  // Hide the label
        } else {
            format_date(&*x)  // Display the date for other points
        }
    })
      .light_line_style(&plotters::prelude::WHITE)
      .x_labels(10)
      .y_labels(5)
      .x_label_style(("sans-serif", 20))
      .y_label_style(("sans-serif", 30))
      .draw()
      .unwrap();

   for s in sensors {
      chart_context
         .draw_series(
            plotters::prelude::LineSeries::new(
               s.curve.clone(),
               plotters::prelude::ShapeStyle {
            color: plotters::prelude::RGBColor(s.colour.0, s.colour.1, s.colour.2).into(),
            filled: false,
            stroke_width: 2,
         },
            )
            .point_size(2),
         )
         .unwrap()
         .label(s.name.clone())
         .legend(|(x, y)| {
            plotters::prelude::PathElement::new(
               vec![(x, y), (x + 20, y)],
               plotters::prelude::RGBColor(s.colour.0, s.colour.1, s.colour.2),
            )
         });
      chart_context.draw_series(std::iter::once(plotters::element::DashedPathElement::new(
         vec![(min_x, s.min), (max_x, s.min)].into_iter(),
         15, // Dash size
         7,  // Gap size
         plotters::prelude::ShapeStyle {
            color: plotters::prelude::RGBColor(s.colour.0, s.colour.1, s.colour.2).into(),
            filled: false,
            stroke_width: 1,
         },
      )))?;
   }

   chart_context
      .configure_series_labels()
      .background_style(&plotters::style::Color::mix(&plotters::style::colors::WHITE, 0.7)) // Translucent white background
      .border_style(&plotters::prelude::RGBColor(211, 211, 211)) // No border
      .label_font(("sans-serif", 20)) // Larger font for labels
      .position(plotters::prelude::SeriesLabelPosition::LowerLeft)
      .draw()?;



   Ok(())
}




//
// ===========================================================================================================
// Tests

#[cfg(test)]
mod tests {
   use super::*;


   fn ts_ymd(year: i32, month: u32, day: u32) -> chrono::DateTime<chrono::Utc> {
      use chrono::TimeZone;
      chrono::Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).earliest().unwrap()
   }

   #[test]
   fn test_plot() {
      let mut sensors = vec![
         Sensor {
            name: "Sensor1".to_string(),
            min: 10.0,
            curve: vec![(ts_ymd(2024, 1, 20), 10.0), (ts_ymd(2024, 1, 21), 13.0)],
            colour: (255, 0, 0),
         },
         Sensor {
            name: "Sensor2".to_string(),
            min: 9.0,
            curve: vec![(ts_ymd(2024, 1, 20), 12.0), (ts_ymd(2024, 1, 21), 14.0)],
            colour: (0, 0, 255),
         },
      ];
      let _ = create_plot(&mut sensors);
   }
}
