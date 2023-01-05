use anyhow::Result;
use std::{sync::mpsc::Sender, thread};

mod ReqResp {
   use anyhow::Result;

   #[derive(Clone)]
   pub struct WakeUp;

   pub struct Reading {
      temperature: Result<f64>,
   }
   impl Reading {
      pub fn new(temperature: Result<f64>) -> Self {
         Reading { temperature }
      }
   }
}

struct SensorPoller {
   reader: Box<dyn sensors::Sensor + std::marker::Send>,
   raid: std::sync::mpsc::Sender<ReqResp::Reading>,
   // remote_endpoint: std::sync::mpsc::Sender<ReqResp::WakeUp>,
   local_endpoint: std::sync::mpsc::Receiver<ReqResp::WakeUp>,
   timer: timer::MessageTimer<ReqResp::WakeUp>,
}

impl SensorPoller {
   fn new(
      reader: Box<dyn sensors::Sensor + std::marker::Send>,
      raid: std::sync::mpsc::Sender<ReqResp::Reading>,
   ) -> Self {
      let (remote_endpoint, local_endpoint) = std::sync::mpsc::channel();
      SensorPoller {
         reader,
         raid,
         local_endpoint,
         timer: timer::MessageTimer::new(remote_endpoint),
      }
   }
   fn start(self) {
      thread::spawn(move || self.event_loop());
   }

   fn event_loop(&self) {
      let mut counter: i32 = 0;
      loop {
         counter += 1;
         let temperature: Result<f64> = self.reader.read();
         let _ = self.raid.send(ReqResp::Reading::new(temperature));
         let _guard = self
            .timer
            .schedule_with_delay(chrono::Duration::seconds(1), ReqResp::WakeUp);
         self.local_endpoint.recv().unwrap();
         println!("This code has been executed after 3 seconds");
         if counter > 10 {
            break;
         }
      }
   }
}

// fn test() {
//    let (tx, rx) = std::sync::mpsc::channel();
//    thread::spawn(move || {
//       let val = String::from("hi");
//       tx.send(val).unwrap();
//    });
// }

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   fn test_sensor_poller() {}
}
