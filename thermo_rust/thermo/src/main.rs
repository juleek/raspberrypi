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
