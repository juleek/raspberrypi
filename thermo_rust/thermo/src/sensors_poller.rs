use crate::sensor_poller;
use crate::sink;
use crossbeam_channel as channel;

struct Wrapper {
   pub name: String,
   pub path: std::path::PathBuf,
   //    pub poller: sensor_poller::SensorPoller,
   pub num_of_readings: i32,
   pub messages: Vec<String>,
   pub temperature: sensors::TempType,
}

impl Wrapper {
   pub fn to_sink_item(&mut self, item: &mut sink::Item) {}
}

pub struct SensorsPoller {
   sink: Box<dyn sink::Sink>,
   wrappers: Vec<Wrapper>,
   local_reading_events: channel::Receiver<sensor_poller::ReqResp::Reading>,
   remote_reading_events: channel::Sender<sensor_poller::ReqResp::Reading>,
   exit_events: channel::Receiver<()>,
}

impl SensorsPoller {
   pub fn new(
      sink: Box<dyn sink::Sink>,
      exit_events: channel::Receiver<()>,
      sensor_infos: std::collections::HashMap<String, std::path::PathBuf>,
   ) -> Self {
      let (remote_reading_events, local_reading_events) = channel::bounded(100);

      let mut wrappers: Vec<Wrapper> = Vec::new();
      for (name, path) in sensor_infos {
         wrappers.push(Wrapper {
            name,
            path,
            num_of_readings: 0,
            messages: Vec::new(),
            temperature: 0.,
         });
      }

      SensorsPoller {
         sink,
         wrappers,
         local_reading_events,
         remote_reading_events,
         exit_events,
      }
   }

   pub fn run(&mut self) {
      for (id, wrapper) in (0i32..).zip(&self.wrappers) {
         let sensor = Box::new(sensors::DS18B20::Sensor::new(id, wrapper.path.to_owned()));
         let poller = sensor_poller::SensorPoller::new(sensor, self.remote_reading_events.clone());
         poller.start();
      }

      loop {
         channel::select! {
             recv(self.local_reading_events) -> reading => {
                 println!("Got a reading: {reading:?}!");
             }
             recv(self.exit_events) -> _ => {
                 println!("Goodbye!");
                 break;
             }
         }
      }
   }
}
