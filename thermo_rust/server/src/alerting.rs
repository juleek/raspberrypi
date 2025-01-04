use anyhow::{anyhow, Context, Result};


// pub struct Alerting {
//    pub name_to_min: std::collections::HashMap<String, f64>,
//    pub sender:      Box<dyn crate::sender::Sender>,
// }

// impl crate::consumer::Consumer for Alerting {
//    fn consume(&self, measurement: helpers::helpers::Measurement) {
//       let mut messages: Vec<String> = Vec::new();
//       for (name, min_temp) in self.name_to_min.iter() {
//          if &measurement.sensor != name {
//             continue;
//          }
//          if let Some(temp) = measurement.temperature {
//             if temp > *min_temp {
//                continue;
//             } else {
//                continue;
//             }
//          }
//          let message = format!("{} is {} degrees, which is {} degrees lower than threshold {}!",
//                                measurement.sensor,
//                                measurement.temperature.unwrap(),
//                                min_temp - measurement.temperature.unwrap(),
//                                min_temp);
//          messages.push(message);
//       }
//       if !messages.is_empty() {
//          let all_messages = &messages.join("\n");
//          self.sender
//              .send_text(all_messages, false)
//              .with_context(|| anyhow!("Failed to send message: {all_messages}"));
//       }
//    }
// }


async fn send_alert_message_if_needed(mut rx: tokio::sync::broadcast::Receiver<agg_proto::MeasurementReq>,
                                   min_temp_bottom: &f64,
                                   min_temp_ambient: &f64,
                                   sender: std::sync::Arc<Box<dyn crate::sender::Sender>>) {
   while let Ok(measurement) = rx.recv().await {
      if let Some(elem) = measurement.measurement {
         let mut messages: Vec<String> = Vec::new();
         for min_temp in [min_temp_bottom, min_temp_ambient] {
            if &elem.temperature.unwrap() < min_temp {
               let message = format!("{} is {} degrees, which is {} degrees lower than threshold {}!",
                                     elem.sensor,
                                     elem.temperature.unwrap(),
                                     min_temp - elem.temperature.unwrap(),
                                     min_temp);
               messages.push(message);
            }
         }
         if !messages.is_empty() {
            let all_messages = messages.join("\n");
            sender.send_text(all_messages, false)
                  .with_context(|| anyhow!("Failed to send message: {all_messages}"))
                  .ok();
         }
      }
   }
}
