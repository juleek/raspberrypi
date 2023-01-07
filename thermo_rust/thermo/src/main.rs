use anyhow::{anyhow, Result};
use crossbeam_channel as channel;
use sensors::DS18B20;
use stdext::function_name;
use thermo::sensor_poller::ReqResp;
use thermo::sensor_poller::SensorPoller;

fn set_ctrl_channel() -> Result<channel::Receiver<()>> {
   let (sender, receiver) = channel::bounded(100);
   match ctrlc::set_handler(move || {
      let _ = sender.send(());
   }) {
      Ok(_) => Ok(receiver),
      Err(ref why) => Err(anyhow!("Failed to {}: {:?}", function_name!(), why)),
   }
}

fn create_poller<'a>(
   path: &std::path::Path,
   remote: channel::Sender<ReqResp::Reading>,
   id: &'a mut i32,
) -> (SensorPoller, &'a mut i32) {
   let reader = Box::new(DS18B20::Sensor::new(*id, std::path::PathBuf::from(path)));
   let poller = SensorPoller::new(reader, remote);
   *id += 1;
   (poller, id)
}

fn main() -> Result<()> {
   let ctrl_c_events = set_ctrl_channel()?;
   let (readings_remote, readings_local) = channel::bounded(100);
   // let ticks = tick(Duration::from_secs(1));

   let mut id = 0;
   let (poller, id) = create_poller(
      std::path::Path::new("/home/dimanne/test.txt"),
      readings_remote,
      &mut id,
   );

   poller.start();

   loop {
      channel::select! {
          recv(readings_local) -> reading => {
              println!("Got a reading: {reading:?}!");
          }
          recv(ctrl_c_events) -> _ => {
              println!("Goodbye!");
              break;
          }
      }
   }

   Ok(())
}
