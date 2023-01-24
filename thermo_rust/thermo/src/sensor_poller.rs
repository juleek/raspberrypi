use crossbeam_channel as channel;
use stdext::function_name;

pub mod ReqResp {
   #[derive(Debug)]
   pub struct Reading(pub sensors::Reading);
}

pub struct SensorPoller {
   reader:   Box<dyn sensors::Sensor + std::marker::Send>,
   raid:     channel::Sender<ReqResp::Reading>,
   to_sleep: std::time::Duration,
}

impl SensorPoller {
   pub fn new(reader: Box<dyn sensors::Sensor + std::marker::Send>,
              raid: channel::Sender<ReqResp::Reading>,
              to_sleep: std::time::Duration)
              -> Self {
      SensorPoller { reader,
                     raid,
                     to_sleep }
   }
   pub fn start(self) { std::thread::spawn(move || self.event_loop()); }

   fn event_loop(&self) {
      loop {
         std::thread::sleep(self.to_sleep);
         //  let timer_channel = channel::after(std::time::Duration::from_secs(1));
         //  channel::select! {
         //     recv(timer_channel) -> _ => (),
         //  }
         let reading = self.reader.read();
         println!("{}: got: {reading:?}", function_name!());
         let _ = self.raid.send(ReqResp::Reading(reading));
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
      let poller = SensorPoller::new(Box::new(sensors::FakeSensor::new(23, 2.5)),
                                     tx,
                                     std::time::Duration::from_secs(1));

      poller.start();

      for received in rx {
         println!("Got: {:?}", received);
      }
   }
}
