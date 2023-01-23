use anyhow::{anyhow, Result};
use clap::Parser;
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

#[derive(clap::Parser)]
struct Cli {
   /// Path of the private key for Google Function
   #[arg(long = "gf/private_key_path")]
   gf_private_key_path: std::path::PathBuf,

   /// Service Account Email
   #[arg(long = "gf/account_email")]
   gf_account_email: String,

   /// Google Function Http end-point
   #[arg(long = "gf/http_end_point")]
   gf_http_end_point: String,

   /// If true we will not publish any data to Google Cloud
   #[arg(long)]
   #[arg(default_value_t = false)]
   dry_run: bool,
}

fn main() -> Result<()> {
   let cli = Cli::parse();

   let ctrl_c_events = set_ctrl_channel()?;

   let factory_bott: thermo::sensors_poller::SensorFactory = Box::new(|id| {
      Box::new(sensors::DS18B20::Sensor::new(
         id,
         // std::path::PathBuf::from("/sys/bus/w1/devices/28-000005eac50a/w1_slave"),
         std::path::PathBuf::from("/home/dimanne/bott.txt"),
      )) as Box<dyn sensors::Sensor + std::marker::Send>
   });
   let factory_amb: thermo::sensors_poller::SensorFactory = Box::new(|id| {
      Box::new(sensors::DS18B20::Sensor::new(
         id,
         // std::path::PathBuf::from("/sys/bus/w1/devices/28-000005eaddc2/w1_slave"),
         std::path::PathBuf::from("/home/dimanne/amb.txt"),
      )) as Box<dyn sensors::Sensor + std::marker::Send>
   });
   let sensors_factories: std::collections::HashMap<String, thermo::sensors_poller::SensorFactory> =
      std::collections::HashMap::from([
         (String::from("BottomTube"), factory_bott),
         (String::from("Ambient"), factory_amb),
      ]);

   let sink = Box::new(thermo::sink::StdOutSink);

   thermo::sensors_poller::run(
      sensors_factories,
      sink,
      ctrl_c_events,
      std::time::Duration::from_secs(1),
   );

   Ok(())
}
