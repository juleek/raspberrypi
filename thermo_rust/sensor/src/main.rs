use anyhow::{anyhow, Context, Result};

async fn async_main(stop_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
                    mut rx: tokio::sync::mpsc::Receiver<sensor::sensor::Measurement>)
                    -> Result<()> {
   // loop {
   //    tokio::select! {

   //        received = rx.recv() => {
   //           println!("received message {:?}", received);
   //     }
   //    }
   // }
   let mut client = agg_proto::agg_client::AggClient::connect("http://127.0.0.1:12345").await.unwrap();
   let req = agg_proto::HelloReq { req: "qwer".to_owned() };
   let resp = client.say_hello(req).await;
   println!("got response: {resp:?}");
   Ok(())
}

fn init_logger(log_level: &str) {
   let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));
   builder.format_timestamp_micros();
   builder.init();
}

fn main() -> Result<()> {
   const INTERVAL: std::time::Duration = std::time::Duration::new(10, 0);
   init_logger("debug");
   let stop_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

   {
      let stop_requested = stop_requested.clone();
      ctrlc::set_handler(move || {
         stop_requested.store(true, std::sync::atomic::Ordering::Relaxed);
      }).map_err(anyhow::Error::new)?;
   }

   let (tx, rx) = tokio::sync::mpsc::channel(10);

   {
      let tx = tx.clone();
      let sensor = sensor::sensor::Sensor { name: "ambient".to_string(),
                                            path: "/home/yulia/devel/test/1".to_string(), };
      let stop_requested_clone = stop_requested.clone();
      std::thread::spawn(move || sensor::sensor::poll_sensor(tx, sensor, stop_requested_clone, INTERVAL));
   }
   {
      let tx = tx.clone();
      let sensor = sensor::sensor::Sensor { name: "bottom".to_string(),
                                            path: "/home/yulia/devel/test/2".to_string(), };
      let stop_requested_clone = stop_requested.clone();
      std::thread::spawn(move || sensor::sensor::poll_sensor(tx, sensor, stop_requested_clone, INTERVAL));
   }
   let rt = tokio::runtime::Builder::new_multi_thread().enable_all()
                                                       .worker_threads(3)
                                                       .thread_name("tokio")
                                                       .build()
                                                       .unwrap();

   rt.block_on(async_main(stop_requested, rx))?;
   Ok(())
}
