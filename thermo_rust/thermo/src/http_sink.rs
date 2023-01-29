use crate::jwt_updater;
use crate::sink;
use crossbeam_channel as channel;


mod req_resp {
   use super::*;
   pub struct Publish(pub sink::Item);
}

struct HttpSinkThread {
   jwt_channel:    channel::Receiver<jwt_updater::req_resp::JWT>,
   pub_channel:    channel::Receiver<req_resp::Publish>,
   http_end_point: String,
   dry_run:        bool,

   last_jwt: String,
   client:   reqwest::blocking::Client,
}
impl HttpSinkThread {
   fn on_new_jwt(&mut self, new_jwt: jwt_updater::req_resp::JWT) {
      log::info!("Got new: {new_jwt:?}");
      self.last_jwt = new_jwt.0;
   }

   fn publish(&self, item: sink::Item) {
      if self.last_jwt.is_empty() {
         log::warn!("Not publishing: {item:?}: jwt token is empty");
         return;
      }

      let req = self.client
                    .post(&self.http_end_point)
                    .header(reqwest::header::AUTHORIZATION,
                            format!("Bearer {}", self.last_jwt))
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(sink::to_json(&item))
                    .build()
                    .expect("Must be possible to create a request");

      log::info!("Making request: {req:?}, data: {:?}", req.body());

      if self.dry_run {
         log::info!("Not publishing item: dry_run mode is enabled");
         return;
      }

      let resp = self.client.execute(req);

      let resp_for_logs = format!("{resp:?}");
      log::info!("Got response: {resp_for_logs}");
   }

   pub fn event_loop(&mut self) {
      log::warn!("event_loop started");
      loop {
         channel::select! {
             recv(self.pub_channel) -> publish_req => {
                match publish_req {
                    Ok(val) => { self.publish(val.0)} ,
                    Err(why) => { log::warn!("Got error: {why}")} ,
                }
             }
             recv(self.jwt_channel) -> new_jwt => {
                match new_jwt {
                    Ok(val) => self.on_new_jwt(val),
                    Err(why) => {log::warn!("Got error: {why}")},
                }
             }
         }
      }
   }
}

pub struct HttpSink {
   pub_channel: channel::Sender<req_resp::Publish>,
}
impl HttpSink {
   pub fn new(jwt_channel: channel::Receiver<jwt_updater::req_resp::JWT>,
              http_end_point: String,
              dry_run: bool)
              -> HttpSink {
      let (sender, receiver) = channel::bounded(100);
      let mut thread = HttpSinkThread { jwt_channel,
                                    pub_channel: receiver,
                                    http_end_point,
                                    dry_run,
                                    last_jwt: Default::default(),
                                    client: reqwest::blocking::Client::builder()
                                    .timeout(std::time::Duration::from_secs(10))
                                    .build()
                                    .expect("Must be possible to create HTTP Client") };
      std::thread::spawn(move || thread.event_loop());
      HttpSink { pub_channel: sender, }
   }
}
impl sink::Sink for HttpSink {
   fn publish(&mut self, item: sink::Item) {
      let _ = self.pub_channel.send(req_resp::Publish(item));
   }
}
