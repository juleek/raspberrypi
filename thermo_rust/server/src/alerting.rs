use anyhow::{Context, Result, anyhow};

pub fn start(
   mut rx: tokio::sync::broadcast::Receiver<common::Measurement>,
   // sensor_db: &crate::sensor::Sqlite,
   sensor_db: std::sync::Arc<dyn crate::sensor::Db>,
   sender: crate::message::Telegram,
) -> Result<()> {
   let sensor_db = sensor_db.clone();
   tokio::task::spawn(async move {
      loop {
         let measurement = match rx.recv().await {
            Ok(msg) => msg,
            // TODO
            Err(_) => continue,
         };
         let res = notify_if_needed(&measurement, &sender, &*sensor_db).await;
         if let Err(why) = res {
            log::warn!("on_cron() failed: {why:?}");
         }
      }
   });

   Ok(())
}

async fn notify_if_needed(
   measurement: &common::Measurement,
   sender: &crate::message::Telegram,
   sensor_db: &dyn crate::sensor::Db,
) -> Result<()> {
   let id = &measurement.id.sensor_id;
   let sensor_meta = sensor_db
      .get_by_id(id)
      .await
      .with_context(|| anyhow!("Failed to sensor_db.get_by_id()"))?;

   let Some(sensor) = sensor_meta else {
      // TODO
      log::warn!("Sensor not found with id: {:?}", id);
      return Ok(());
   };
   let min_threshold = sensor.min;

   // measurement.temperature.ok_or_else()
   let temperature = match measurement.temperature {
      Some(temp) => temp,
      None => return Ok(()),
   };

   if temperature <= min_threshold {
      let difference = min_threshold - temperature;
      let text = format!(
         "{} temperature is {} degrees, which is {} lower than threshold {}!",
         sensor.name, temperature, difference, min_threshold
      );
      sender.send_text(text, false).await?;
   }
   Ok(())
}

// mockall

// fn test {
//    let mock_messenger =
// }
