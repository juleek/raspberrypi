use crate::sensor_poller;
use crate::sink;
use crossbeam_channel as channel;

pub type SensorFactory = Box<dyn FnOnce(i32) -> Box<dyn sensors::Sensor + std::marker::Send>>;

struct Wrapper {
   pub name: String,
   pub num_of_readings: i32,
   pub messages: Vec<String>,
   pub temperature: sensors::TempType,
}

impl Wrapper {
   pub fn to_sink_item(&mut self, item: &mut sink::Item) {}
}

pub fn run(
   sensor_factories: std::collections::HashMap<String, SensorFactory>,
   sink: Box<dyn sink::Sink>,
   exit_events: channel::Receiver<()>,
) {
   let (remote_reading_events, local_reading_events) = channel::bounded(100);
   let mut wrappers: Vec<Wrapper> = Vec::new();
   for (id, (name, factory)) in (0i32..).zip(sensor_factories) {
      wrappers.push(Wrapper {
         name,
         num_of_readings: 0,
         messages: Vec::new(),
         temperature: 0.,
      });
      let poller = sensor_poller::SensorPoller::new(factory(id), remote_reading_events.clone());
      poller.start();
   }

   loop {
      channel::select! {
          recv(local_reading_events) -> reading => {
              println!("Got a reading: {reading:?}!");
          }
          recv(exit_events) -> _ => {
              println!("Goodbye!");
              break;
          }
      }
   }
}
