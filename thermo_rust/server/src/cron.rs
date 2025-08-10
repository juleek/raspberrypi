use anyhow::{Context, Result, anyhow};

fn generate_candidates(date: chrono::NaiveDate) -> Vec<chrono::DateTime<chrono::Utc>> {
   use chrono::TimeZone;
   let tz = chrono_tz::Europe::Moscow;
   let times = [
      chrono::NaiveTime::from_hms_opt(1, 55, 0).unwrap(),
      chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
      chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
      chrono::NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
   ];

   times
      .iter()
      .filter_map(|&nt| tz.from_local_datetime(&date.and_time(nt)).latest())
      .map(|dt| dt.with_timezone(&chrono::Utc))
      .collect()
}

fn calculate_next_run_time(now: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
   let candidates = generate_candidates(now.date_naive());
   if let Some(next) = candidates.iter().find(|c| **c > now) {
      return *next;
   }
   let tomorrow = (now + chrono::Duration::days(1)).date_naive();
   let candidates = generate_candidates(tomorrow);
   candidates.first().copied().unwrap_or(now + chrono::Duration::hours(1))
}

pub fn start(
   measurements_db: &crate::db::measurement::Sqlite,
   sensor_db: &crate::sensor::Sqlite,
   sender: crate::message::Telegram,
) -> Result<()> {
   return Ok(());

   // tokio::task::spawn({
   //    let measurements_db = measurements_db.clone();
   //    let sensor_db = sensor_db.clone();
   //    async move {
   //       loop {
   //          let now = chrono::Utc::now();
   //          let next_run = calculate_next_run_time(now);
   //          let to_sleep = (next_run - now).to_std().unwrap_or(std::time::Duration::ZERO);
   //          log::info!("now: {now} => sleeping {} until {next_run}", human_duration::human_duration(&to_sleep));
   //          tokio::time::sleep(to_sleep).await;
   //          let res = on_cron(&sender, &sensor_db, &measurements_db).await;
   //          if let Err(why) = res {
   //             log::warn!("on_cron() failed: {why:?}")
   //          }
   //       }
   //    }
   // });
   // Ok(())
}


async fn on_cron(
   sender: &crate::message::Telegram,
   sensor_db: &crate::sensor::Sqlite,
   measurements_db: &crate::db::measurement::Sqlite,
) -> Result<()> {
   let now = chrono::Utc::now();
   let start = common::MicroSecTs(now - chrono::Duration::hours(24));
   let end = common::MicroSecTs(now);

   use crate::db::measurement::Db as _;
   use crate::sensor::Db as _;

   let sensors_meta = sensor_db.get_all().await.with_context(|| anyhow!("Failed to sensor_db.get_all()"))?;
   let mut plot_sensors: Vec<crate::plot::Sensor> = Vec::new();
   let colours = vec![(255, 0, 0), (0, 0, 255), ((0, 255, 0))];
   let mut errors: Vec<String> = Vec::new();
   for (i, sensor_meta) in sensors_meta.iter().enumerate() {
      let measurements = measurements_db.read(start, end, &sensor_meta.id).await.with_context(|| {
         anyhow!("Failed to read measurements from {start:?} until {end:?} of sensor: {sensor_meta:?}")
      })?;
      let curve: Vec<_> = measurements
         .clone()
         .into_iter()
         .filter_map(|measurement| measurement.temperature.map(|temp| (measurement.read_ts.0, temp)))
         .collect();

      let curve_errors: Vec<&str> = measurements
         .iter()
         .filter(|error| !error.error.is_empty())
         .take(10)
         .map(|s| s.error.as_str())
         .collect();
      let curve_errors = curve_errors.join(", ");
      plot_sensors.push(crate::plot::Sensor {
         name: sensor_meta.clone().name,
         min: sensor_meta.min,
         curve,
         colour: colours[i % colours.len()],
      });
      errors.push(format!("sensor: {curve_errors}"));
   }
   let errors = errors.join("\n");
   let plot = crate::plot::create_plot(&mut plot_sensors)?;
   sender.send_with_pic(&errors, plot).await?;
   Ok(())
}
