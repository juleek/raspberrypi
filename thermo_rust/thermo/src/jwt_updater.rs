use anyhow::Result;
use crossbeam_channel as channel;

pub mod ReqResp {
   #[derive(Debug)]
   pub struct JWT(pub String);
}

pub struct JwtUpdater {
   client: reqwest::blocking::Client,
   raid: channel::Sender<ReqResp::JWT>,
}

impl JwtUpdater {
   pub fn new(raid: channel::Sender<ReqResp::JWT>) -> Self {
      JwtUpdater {
         raid,
         client: reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Must be possible to create HTTP Client"),
      }
   }
   pub fn start(self) {
      std::thread::spawn(move || self.event_loop());
   }

   fn event_loop(&self) {
      let mut counter: i32 = 0;
      loop {
         counter += 1;

         let resp = self.client.get("https://google.com").send();
         println!("{resp:#?}");

         std::thread::sleep(std::time::Duration::from_secs(1));
        //  let timer_channel = channel::after(std::time::Duration::from_secs(1));
        //  channel::select! {
        //     recv(timer_channel) -> _ => (),
        //  }
         // let _ = self.raid.send(ReqResp::Reading(temperature));
         if counter > 3 {
            break;
         }
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_updater() {
      let (tx, rx) = channel::bounded(100);
      // let sensor = sensors::FakeSensor::new(23, 2.5);
      let poller = JwtUpdater::new(tx);

      poller.start();

      for received in rx {
         println!("Got: {:?}", received);
      }
   }
}
