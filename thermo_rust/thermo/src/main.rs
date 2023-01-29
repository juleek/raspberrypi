use anyhow::{anyhow, Result};
use clap::Parser;
use crossbeam_channel as channel;
// use std::io::Write;
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
   publish_dry_run: bool,

   /// Log-level. We are using env_logger, so everything can be configured with env-vars, but we also
   /// provide an option to configure it with a command-line arguments.
   /// Valid values are error, warn, info, debug, trace.
   /// See more info at https://docs.rs/env_logger/0.10.0/env_logger/
   #[arg(long)]
   #[arg(default_value_t = String::from("info"))]
   log_level: String,
}

fn main() -> Result<()> {
   let cli: Cli = Cli::parse();
   env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(cli.log_level))
   .format_timestamp_micros()
//    .format(|buf, record| {
//       writeln!(
//           buf,
//           "{}:{} {} [{}] - {}",
//           record.file().unwrap_or("unknown"),
//           record.line().unwrap_or(0),
//           chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
//           record.level(),
//           record.args()
//       )
//   })
  .init();
   let ctrl_c_events = set_ctrl_channel()?;

   // Set up JWT Updater:
   let private_key = std::fs::read_to_string(cli.gf_private_key_path)?;
   let (jwt_sender_channel, jwt_receiver_channel) = channel::bounded(100);
   let jwt_updater = thermo::jwt_updater::JwtUpdater::new(jwt_sender_channel,
                                                          &cli.gf_http_end_point,
                                                          &cli.gf_account_email,
                                                          &private_key);
   jwt_updater.start();


   // Set up Sink:
   // let mut sink = thermo::sink::StdOutSink;
   let mut sink = thermo::http_sink::HttpSink::new(jwt_receiver_channel,
                                                   cli.gf_http_end_point,
                                                   cli.publish_dry_run);


   // Setup the main driver:
   let mut sensors_factories = std::collections::HashMap::new();
   for (name, path) in [("BottomTube", "/sys/bus/w1/devices/28-000005eac50a/w1_slave"),
                        ("Ambient", "/sys/bus/w1/devices/28-000005eaddc2/w1_slave")]
   {
      let path = std::path::PathBuf::from(path);
      let factory: thermo::sensors_poller::SensorFactory =
         Box::new(|id| Box::new(sensors::DS18B20::Sensor::new(id, path)));
      sensors_factories.insert(String::from(name), factory);
   }
   thermo::sensors_poller::run(sensors_factories,
                               &mut sink,
                               ctrl_c_events,
                               std::time::Duration::from_secs(1));

   Ok(())
}
