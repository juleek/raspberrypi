use anyhow::{anyhow, Context, Result};

// use agg_proto::agg_client::AggClient;
// use async_stream::stream;
// use futures::{Stream, StreamExt}; // Ensure Stream is imported
// use std::time::{Duration, Instant};
// use tokio::time;
// use tonic::Request;

// async fn async_main(stop_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
//                     mut rx: sensor::sensor::Rx)
//                     -> Result<()> {
//    let mut client = agg_proto::agg_client::AggClient::connect("http://127.0.0.1:12345").await.unwrap();
//    let start = Instant::now();
//    let outbound = stream! {
//        let mut interval = time::interval(Duration::from_secs(1));
//        while let _ = interval.tick().await {
//            let elapsed = start.elapsed(); // Use elapsed properly if needed
//            let req = agg_proto::MeasurementReq { measurement: None, counter: 123 };
//            yield req;
//        }
//    };

//    let response = client.send_measurement(outbound).await?;
//    let mut inbound = response.into_inner();

//    while let Some(counter) = inbound.message().await? {
//       // tx_ack.send(counter).await?;
//    }

//    Ok(())
// }

use tokio::sync::mpsc;
use tokio_stream::StreamExt; // Provides the `next` method for streams
// use futures::stream; // To create example streams

async fn async_main(stop_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
                    mut thread_rx: sensor::sensor::Rx)
                    -> Result<()> {
   let mut client = agg_proto::agg_client::AggClient::connect("http://127.0.0.1:12345").await.unwrap();


   let (tx_outbound, rx_outbound) = tokio::sync::mpsc::channel(10);
   let outbound = tokio_stream::wrappers::ReceiverStream::new(rx_outbound);
   let mut inbound_stream = client.send_measurement(outbound).await?.into_inner();

   loop {
      tokio::select! {
         Some(measurement) = thread_rx.recv() => {
            // ts += 1;
            let req = agg_proto::MeasurementReq { measurement: Some(measurement.into()), counter: 123, };
            println!("Client sending: {:?}", req);

            if tx_outbound.send(req).await.is_err() {
               println!("Error");
               break;
           }
         },
         response = inbound_stream.message() => {
            println!("Client received from server: {:?}", response);
         },

         }
      }

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
