use crate::sensor_poller;
use crate::sink;
use crossbeam_channel as channel;

pub type SensorFactory = Box<dyn FnOnce(i32) -> Box<dyn sensors::Sensor + std::marker::Send>>;

struct Wrapper {
   name:            String,
   num_of_readings: i32,
   temperature:     sensors::TempType,
   messages:        Vec<String>,
}

impl Wrapper {
   pub fn new(name: String) -> Wrapper {
      Wrapper { name,
                num_of_readings: 0,
                messages: Vec::new(),
                temperature: 0. }
   }

   // Read-only:
   pub fn name(&self) -> &str { &self.name }
   pub fn num_of_readings(&self) -> i32 { self.num_of_readings }
   pub fn move_state_to_sink_item(&mut self, item: &mut sink::Item) {
      self.num_of_readings = 0;
      if self.messages.is_empty() == false {
         let sep = if item.ErrorString.is_empty() {
            ""
         } else {
            "\n"
         };
         item.ErrorString += &(sep.to_owned() + &std::mem::take(&mut self.messages).join(", "));
      }
      item.NameToTemp.insert(self.name().to_string(),
                             std::mem::take(&mut self.temperature));
   }

   // Populating:
   pub fn on_new_temperature_got(&mut self, reading: sensor_poller::req_resp::Reading) {
      log::info!("Sensor: {}, got a: {reading:?}", self.name);
      self.num_of_readings += 1;
      match reading.0.measurement {
         Ok(val) => self.temperature = val,
         Err(why) => self.messages.push(format!("{why}")),
      };
   }
}

fn on_new_temperature_got(sink: &mut dyn sink::Sink,
                          wrappers: &mut [Wrapper],
                          reading: sensor_poller::req_resp::Reading) {
   wrappers[reading.0.id as usize].on_new_temperature_got(reading);
   let max = wrappers.iter()
                     .max_by_key(|e| e.num_of_readings)
                     .expect("Must be non empty");
   let min = wrappers.iter()
                     .min_by_key(|e| e.num_of_readings)
                     .expect("Must be non empty");
   const MAX_DIFFERENCE_BETWEEN_SENSORS: i32 = 4;
   if min.num_of_readings() == 0 && max.num_of_readings() < MAX_DIFFERENCE_BETWEEN_SENSORS {
      // We know that there is at least one lagging sensor (Min)
      // but the diff between it and the most advanced one is less than the threshold => we are can wait more
      return;
   }

   let mut sink_item = sink::Item::new_with_curr_time();
   if min.num_of_readings() == 0 {
      // max.num_of_readings() >= MAX_DIFFERENCE_BETWEEN_SENSORS
      sink_item.ErrorString = format!(
         "We were able to read {} times from sensor {}, but were unable to read once from sensor {}",
         max.num_of_readings(),
         max.name(),
         min.name()
      );
   }
   for w in wrappers.iter_mut().filter(|w| w.num_of_readings() != 0) {
      w.move_state_to_sink_item(&mut sink_item);
   }

   log::info!("Publishing: {sink_item:?}");

   sink.publish(sink_item);
}

pub fn run(sensor_factories: std::collections::HashMap<String, SensorFactory>,
           sink: &mut dyn sink::Sink,
           exit_events: channel::Receiver<()>,
           sensor_polling_freq: std::time::Duration) {
   let (remote_reading_events, local_reading_events) = channel::bounded(100);
   let mut wrappers: Vec<Wrapper> = Vec::new();

   for (id, (name, factory)) in (0i32..).zip(sensor_factories) {
      log::warn!("Sensor id: {id}, name: {name}");
      wrappers.push(Wrapper::new(name));
      let poller = sensor_poller::SensorPoller::new(factory(id),
                                                    remote_reading_events.clone(),
                                                    sensor_polling_freq);
      poller.start();
   }

   loop {
      channel::select! {
          recv(local_reading_events) -> reading => {
            on_new_temperature_got(sink, &mut wrappers, reading.expect("Must be possible to send messages via MessagePassing framework"));
          }
          recv(exit_events) -> _ => {
              log::warn!("Ctrl+C pressed: exiting...");
              break;
          }
      }
   }
}




#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   struct MockSensorFactory {
      pub ctrlc_sender: channel::Sender<()>,
   }

   impl FnOnce<(i32,)> for MockSensorFactory {
      type Output = Box<dyn sensors::Sensor + std::marker::Send>;
      // This is a callable that gets id and returns unique_ptr<MockSensor>:
      extern "rust-call" fn call_once(self,
                                      (id,): (i32,))
                                      -> Box<dyn sensors::Sensor + std::marker::Send> {
         let mut counter1 = -1;
         Box::new(sensors::MockSensor::new(id, // MockSensor is initialised with id:
                                           // And read callback. RefCell for interior mutability. Box::new(|| ...) is std::function
                                           std::cell::RefCell::new(Box::new(move || {
                                              counter1 += 1; // The read callback will increment the counter:
                                              if counter1 == 5 {
                                                 let _ = self.ctrlc_sender.send(());
                                              }
                                              // and return a reading:
                                              sensors::Reading { measurement:
                                                                    Ok(counter1
                                                                       as sensors::TempType),
                                                                 id }
                                           })))) //  as Box<dyn sensors::Sensor + std::marker::Send>
      }
   }

   #[test]
   fn single_sensor_check_data_provided_by_sensor_is_published() {
      // -----------------------------------------------------------------------------------------------------
      // Setup
      let (ctrlc_sender, ctrlc_receiver): (channel::Sender<()>, channel::Receiver<()>) =
         channel::bounded(100);

      const F1_NAME: &str = "Sensor:BottomTube";
      let f1: SensorFactory = Box::new(MockSensorFactory { ctrlc_sender: ctrlc_sender.clone(), });

      let factories: std::collections::HashMap<String, SensorFactory> =
         std::collections::HashMap::from([(String::from(F1_NAME), f1)]);

      let mut sink = sink::FakeSink::default();

      // -----------------------------------------------------------------------------------------------------
      // Run test:
      run(factories,
          &mut sink,
          ctrlc_receiver,
          std::time::Duration::from_millis(1));

      // -----------------------------------------------------------------------------------------------------
      // Check results:
      assert!(!sink.items.is_empty());
      for (i, item) in sink.items.iter().enumerate() {
         assert!(item.NameToTemp.contains_key(F1_NAME));
         assert_eq!(*item.NameToTemp.get(F1_NAME).unwrap(),
                    i as sensors::TempType);
         assert!(item.ErrorString.is_empty());
      }
   }

   #[test]
   fn two_sensors_check_data_provided_by_sensor_is_published() {
      // -----------------------------------------------------------------------------------------------------
      // Setup
      let (ctrlc_sender, ctrlc_receiver): (channel::Sender<()>, channel::Receiver<()>) =
         channel::bounded(100);
      const F1_NAME: &str = "Sensor:BottomTube";
      let f1: SensorFactory = Box::new(MockSensorFactory { ctrlc_sender: ctrlc_sender.clone(), });
      const F2_NAME: &str = "Sensor:Ambient";
      let f2: SensorFactory = Box::new(MockSensorFactory { ctrlc_sender: ctrlc_sender.clone(), });
      let factories: std::collections::HashMap<String, SensorFactory> =
         std::collections::HashMap::from([(String::from(F1_NAME), f1),
                                          (String::from(F2_NAME), f2)]);
      let mut sink = sink::FakeSink::default();

      // -----------------------------------------------------------------------------------------------------
      // Run test:
      run(factories,
          &mut sink,
          ctrlc_receiver,
          std::time::Duration::from_millis(1));

      // -----------------------------------------------------------------------------------------------------
      // Check results:
      assert!(!sink.items.is_empty());
      for (i, item) in sink.items.iter().enumerate() {
         assert!(item.NameToTemp.contains_key(F1_NAME));
         assert!(item.NameToTemp.contains_key(F2_NAME));
         assert_eq!(*item.NameToTemp.get(F1_NAME).unwrap(),
                    i as sensors::TempType);
         assert_eq!(*item.NameToTemp.get(F2_NAME).unwrap(),
                    i as sensors::TempType);
         assert!(item.ErrorString.is_empty());
      }
   }

   #[test]
   fn two_sensors_one_of_them_is_slow_check_error_is_reported() {
      // -----------------------------------------------------------------------------------------------------
      // Setup
      let (ctrlc_sender, ctrlc_receiver): (channel::Sender<()>, channel::Receiver<()>) =
         channel::bounded(100);
      const F1_NAME: &str = "Sensor:BottomTube";
      let f1: SensorFactory = Box::new(MockSensorFactory { ctrlc_sender: ctrlc_sender.clone(), });
      const F2_NAME: &str = "Sensor:Ambient";
      let f2: crate::sensors_poller::SensorFactory = Box::new(move |id| {
         Box::new(sensors::MockSensor::new(id,
                                           std::cell::RefCell::new(Box::new(move || {
                                              std::thread::sleep(std::time::Duration::MAX);
                                              sensors::Reading { measurement: Ok(0.),
                                                                 id }
                                           }))))
         as Box<dyn sensors::Sensor + std::marker::Send>
      });
      let factories: std::collections::HashMap<String, SensorFactory> =
         std::collections::HashMap::from([(String::from(F1_NAME), f1),
                                          (String::from(F2_NAME), f2)]);
      let mut sink = sink::FakeSink::default();

      // -----------------------------------------------------------------------------------------------------
      // Run test:
      run(factories,
          &mut sink,
          ctrlc_receiver,
          std::time::Duration::from_millis(1));

      // -----------------------------------------------------------------------------------------------------
      // Check results:
      // Has to report an error like:
      // [Item { NameToTemp: {"Sensor:BottomTube": 3.0}, ErrorString: "We were able to read 4 times from sensor
      // Sensor:BottomTube, but were unable to read once from sensor Sensor:Ambient" }]
      assert!(sink.items[0].ErrorString.contains(F2_NAME));
      assert!(sink.items[0].NameToTemp.contains_key(F1_NAME));
   }
}
