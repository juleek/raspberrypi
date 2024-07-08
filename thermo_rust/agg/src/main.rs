use anyhow::{anyhow, Context, Result};

struct Agg {
   counter: std::sync::Mutex<i32>,
}

#[tonic::async_trait]
impl agg_proto::agg_server::Agg for Agg {
   async fn say_hello(&self,
                      request: tonic::Request<agg_proto::HelloReq>)
                      -> Result<tonic::Response<agg_proto::HelloResp>, tonic::Status> {
      let counter = {
         let mut counter = self.counter.lock().unwrap();
         *counter += 1;
         *counter
      };

      println!("function is called with: {request:?}");
      let resp = agg_proto::HelloResp {resp: counter};
      return Ok(tonic::Response::new(resp))
   }
}

async fn async_main() -> Result<()> {
   let agg = Agg { counter: std::sync::Mutex::new(10) };
   let agg = agg_proto::agg_server::AggServer::new(agg);

   tonic::transport::Server::builder().add_service(agg)
                                      .serve("0.0.0.0:12345".parse().unwrap())
                                      .await?;
   Ok(())
}

fn init_logger(log_level: &str) {
   let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));
   builder.format_timestamp_micros();
   builder.init();
}

fn main() -> Result<()> {
   init_logger("debug");
   let stop_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

   {
      let stop_requested = stop_requested.clone();
      ctrlc::set_handler(move || {
         stop_requested.store(true, std::sync::atomic::Ordering::Relaxed);
      }).map_err(anyhow::Error::new)?;
   }
   let rt = tokio::runtime::Builder::new_multi_thread().enable_all()
                                                       .worker_threads(3)
                                                       .thread_name("tokio")
                                                       .build()
                                                       .unwrap();

   rt.block_on(async_main())?;
   Ok(())
}
