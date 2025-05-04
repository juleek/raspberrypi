use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug)]
struct PlotLine {
   legend: String,
   colour: plotters::prelude::RGBColor,
   x: Vec<chrono::DateTime<chrono::Utc>>,
   y: Vec<f64>,
   threshold_hline: Option<f64>,
}

#[derive(Debug)]
struct PlotInfo {
   title: String,
   lines: Vec<PlotLine>,
   dpi: u32,
   title_font_size: u32,
   legend_font_size: u32,
   format: String,
}

fn make_plot(plot_info: &PlotInfo) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
   use plotters::drawing::IntoDrawingArea;
   let root = plotters::prelude::BitMapBackend::new("plot.png", (700, 700)).into_drawing_area();
   root.fill(&plotters::prelude::WHITE)?;

   let (min_x, max_x) = plot_info.lines.iter().flat_map(|line| line.x.iter()).fold(
      (None::<f64>, None::<f64>),
      |(min, max), dt| {
         let ts = dt.timestamp() as f64;
         (Some(min.map_or(ts, |m| m.min(ts))), Some(max.map_or(ts, |m| m.max(ts))))
      },
   );

   let (min_y, max_y) = plot_info
      .lines
      .iter()
      .flat_map(|line| line.y.iter())
      .fold((None::<f64>, None::<f64>), |(min, max), &val| {
         (Some(min.map_or(val, |m| m.min(val))), Some(max.map_or(val, |m| m.max(val))))
      });

   let min_x = min_x.unwrap_or(0.0);
   let max_x = max_x.unwrap_or(1.0);
   let min_y = min_y.unwrap_or(0.0);
   let max_y = max_y.unwrap_or(1.0);

   let mut chart = plotters::prelude::ChartBuilder::on(&root)
      .caption(&plot_info.title, ("sans-serif", plot_info.title_font_size))
      .margin(20)
      .x_label_area_size(plot_info.legend_font_size)
      .y_label_area_size(plot_info.legend_font_size)
      .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

   // chart.configure_mesh().x_labels(4).y_labels(4).draw()?;
   chart
      .configure_mesh()
      .light_line_style(&plotters::prelude::WHITE)
      .x_label_style(("sans-serif", 30))
      .y_label_style(("sans-serif", 30))
      .draw()?;

   use plotters::style::Color;
   for line in &plot_info.lines {
      let series: Vec<_> = line.x.iter().zip(&line.y).map(|(dt, &y)| (dt.timestamp() as f64, y)).collect();

      chart
         .draw_series(plotters::prelude::LineSeries::new(
            series,
            plotters::prelude::ShapeStyle {
               color: line.colour.to_rgba(),
               filled: false,
               stroke_width: 5,
            },
         ))?
         .label(&line.legend)
         .legend(move |(x, y): (i32, i32)| {
            let legend_path = vec![(x - 10, y), (x + 10, y)];
            plotters::element::DashedPathElement::new(
               legend_path.into_iter(),
               5,
               3,
               plotters::prelude::ShapeStyle {
                  color: line.colour.to_rgba(),
                  filled: false,
                  stroke_width: 2,
               },
            )
         });

      if let Some(threshold) = line.threshold_hline {
         chart.draw_series(std::iter::once(plotters::element::DashedPathElement::new(
            vec![(min_x, threshold), (max_x, threshold)].into_iter(),
            30, // Dash size
            7,  // Gap size
            plotters::prelude::ShapeStyle {
               color: line.colour.to_rgba(),
               filled: false,
               stroke_width: 2,
            },
         )))?;
      }
   }

   chart
      .configure_series_labels()
      .background_style(&plotters::prelude::WHITE.mix(0.9)) // Translucent white background
      .border_style(&plotters::prelude::RGBColor(211, 211, 211)) // No border
      .label_font(("Arial", 20)) // Larger font for labels
      .position(plotters::prelude::SeriesLabelPosition::LowerLeft)
      .draw()?;
   root.present()?;

   let mut buf = Vec::new();
   let mut file = std::fs::File::open("plot.png")?;
   use std::io::Read;
   file.read_to_end(&mut buf)?;
   Ok(buf)
}

pub fn create_plot(min_temp_bottom: &f64, min_temp_ambient: &f64) -> Result<(), Box<dyn std::error::Error>> {
   let bottom_tube_line = PlotLine {
      legend: "BottomTube".to_string(),
      colour: plotters::prelude::RED,
      x: vec![chrono::Utc.timestamp(0, 0), chrono::Utc.timestamp(10, 0), chrono::Utc.timestamp(20, 0)],
      y: vec![10.0, 20.0, 15.0],
      threshold_hline: Some(*min_temp_bottom),
   };

   let ambient_line = PlotLine {
      legend: "Ambient".to_string(),
      colour: plotters::prelude::BLUE,
      x: vec![chrono::Utc.timestamp(0, 0), chrono::Utc.timestamp(10, 0), chrono::Utc.timestamp(20, 0)],
      y: vec![5.0, 10.0, 7.0],
      threshold_hline: Some(*min_temp_ambient),
   };

   let current_time = Utc::now().with_timezone(&chrono_tz::Europe::Moscow).format("%d.%m  %H:%M").to_string();

   let plot_info = PlotInfo {
      title: format!("Temp in Tarasovka on {}", current_time),
      lines: vec![bottom_tube_line, ambient_line],
      dpi: 500,
      title_font_size: 40,
      legend_font_size: 40,
      format: "png".to_string(),
   };

   let png_data = make_plot(&plot_info)?;
   println!("Plot created successfully with size: {} bytes", png_data.len());
   Ok(())
}



//
// ===========================================================================================================
// Tests

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_plot() {
      let min_temp_bottom = 15.0;
      let min_temp_ambient = 6.0;
      let result = create_plot(&min_temp_bottom, &min_temp_ambient);
   }
}
