use anyhow::Result;
use crossbeam_channel as channel;

pub mod ReqResp {
   #[derive(Debug)]
   pub struct Reading(pub sensors::Reading);
}

pub struct SensorPoller {
   reader: Box<dyn sensors::Sensor + std::marker::Send>,
   raid: channel::Sender<ReqResp::Reading>,
}

impl SensorPoller {
   pub fn new(
      reader: Box<dyn sensors::Sensor + std::marker::Send>,
      raid: channel::Sender<ReqResp::Reading>,
   ) -> Self {
      SensorPoller { reader, raid }
   }
   pub fn start(self) {
      std::thread::spawn(move || self.event_loop());
   }

   fn event_loop(&self) {
      loop {
         std::thread::sleep(std::time::Duration::from_secs(1));
         //  let timer_channel = channel::after(std::time::Duration::from_secs(1));
         //  channel::select! {
         //     recv(timer_channel) -> _ => (),
         //  }
         let temperature: Result<f64> = self.reader.read();
         println!("got: {temperature:?}");
         let _ = self.raid.send(ReqResp::Reading(temperature));
      }
   }
}

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   #[test]
   #[ignore = "Integration test: Uses sleep()"]
   fn test_sensor_poller() {
      let (tx, rx) = channel::bounded(100);
      // let sensor = sensors::FakeSensor::new(23, 2.5);
      let poller = SensorPoller::new(Box::new(sensors::FakeSensor::new(23, 2.5)), tx);

      poller.start();

      for received in rx {
         println!("Got: {:?}", received);
      }
   }
}
