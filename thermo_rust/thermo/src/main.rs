use anyhow::{anyhow, Result};
use crossbeam_channel as channel;
use stdext::function_name;

fn set_ctrl_channel() -> Result<channel::Receiver<()>> {
   let (sender, receiver) = channel::bounded(100);
   match ctrlc::set_handler(move || {
      let _ = sender.send(());
   }) {
      Ok(_) => Ok(receiver),
      Err(ref why) => Err(anyhow!("Failed to {}: {:?}", function_name!(), why)),
   }
}

fn main() -> Result<()> {
   let ctrl_c_events = set_ctrl_channel()?;

   let sensors_info = std::collections::HashMap::from([
      (
         String::from("BottomTube"),
         //  std::path::PathBuf::from("/sys/bus/w1/devices/28-000005eac50a/w1_slave"),
         std::path::PathBuf::from("/home/dimanne/bott.txt"),
      ),
      (
         String::from("Ambient"),
         //  std::path::PathBuf::from("/sys/bus/w1/devices/28-000005eaddc2/w1_slave"),
         std::path::PathBuf::from("/home/dimanne/amb.txt"),
      ),
   ]);

   let sink = Box::new(thermo::sink::StdOutSink);

   let mut sensors_poller =
      thermo::sensors_poller::SensorsPoller::new(sink, ctrl_c_events, sensors_info);
   sensors_poller.run();

   Ok(())
}
