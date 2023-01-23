use crate::sensor_poller;
use crate::sink;
use crossbeam_channel as channel;

pub type SensorFactory = Box<dyn FnOnce(i32) -> Box<dyn sensors::Sensor + std::marker::Send>>;

struct Wrapper {
   name: String,
   num_of_readings: i32,
   temperature: sensors::TempType,
   messages: Vec<String>,
}

impl Wrapper {
   pub fn new(name: String) -> Wrapper {
      Wrapper {
         name,
         num_of_readings: 0,
         messages: Vec::new(),
         temperature: 0.,
      }
   }

   // Read-only:
   pub fn name(&self) -> &str {
      &self.name
   }
   pub fn num_of_readings(&self) -> i32 {
      self.num_of_readings
   }
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
      item.NameToTemp.insert(
         self.name().to_string(),
         std::mem::take(&mut self.temperature),
      );
   }

   // Populating:
   pub fn on_new_temperature_got(&mut self, reading: sensor_poller::ReqResp::Reading) {
      // println!("Got a reading: {reading:?}!");
      self.num_of_readings += 1;
      match reading.0.measurement {
         Ok(val) => self.temperature = val,
         Err(why) => self.messages.push(format!("{why}")),
      };
   }
}

fn on_new_temperature_got(
   sink: &mut dyn sink::Sink,
   wrappers: &mut [Wrapper],
   reading: sensor_poller::ReqResp::Reading,
) {
   wrappers[reading.0.id as usize].on_new_temperature_got(reading);
   let max = wrappers
      .iter()
      .max_by_key(|e| e.num_of_readings)
      .expect("Must be non empty");
   let min = wrappers
      .iter()
      .min_by_key(|e| e.num_of_readings)
      .expect("Must be non empty");
   const MAX_DIFFERENCE_BETWEEN_SENSORS: i32 = 4;
   if min.num_of_readings() == 0 && max.num_of_readings() < MAX_DIFFERENCE_BETWEEN_SENSORS {
      // We know that there is at least one lagging sensor (Min)
      // but the diff between it and the most advanced one is less than the threshold => we are can wait more
      return;
   }

   let mut sink_item = sink::Item::default();
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
   sink.publish(sink_item);
}

pub fn run(
   sensor_factories: std::collections::HashMap<String, SensorFactory>,
   sink: &mut dyn sink::Sink,
   exit_events: channel::Receiver<()>,
   sensor_polling_freq: std::time::Duration,
) {
   let (remote_reading_events, local_reading_events) = channel::bounded(100);
   let mut wrappers: Vec<Wrapper> = Vec::new();

   for (id, (name, factory)) in (0i32..).zip(sensor_factories) {
      wrappers.push(Wrapper::new(name));
      let poller = sensor_poller::SensorPoller::new(
         factory(id),
         remote_reading_events.clone(),
         sensor_polling_freq,
      );
      poller.start();
   }

   // let s = &mut *sink;

   loop {
      channel::select! {
          recv(local_reading_events) -> reading => {
            on_new_temperature_got(sink, &mut wrappers, reading.expect("Must be possible to send messages via MessagePassing framework"));
          }
          recv(exit_events) -> _ => {
              println!("Goodbye!");
              break;
          }
      }
   }
}

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   #[test]
   fn single_sensor_check_data_provided_by_sensor_is_published() {
      // -----------------------------------------------------------------------------------------------------
      // Setup

      let (ctrlc_sender, ctrlc_receiver): (channel::Sender<()>, channel::Receiver<()>) =
         channel::bounded(100);
      let mut counter = std::sync::Arc::new(std::sync::atomic::AtomicI32::default());

      let mut ctrlc_sender1 = ctrlc_sender.clone();
      let mut counter1 = counter.clone();

      // let callback1: Box<dyn FnMut() -> sensors::Reading + Send + Sync> = ;
      let factory1: crate::sensors_poller::SensorFactory = Box::new(|id| {
         // This is a lambda that get id and returns unique_ptr<MockSensor>:
         Box::new(sensors::MockSensor::new(
            // MockSensor is initialised with id:
            id,
            // And read callback. RefCell for interior mutability. Box::new(|| ...) is std::function
            std::cell::RefCell::new(Box::new(move || {
               // println!("In the read lambda!");
               // The read callback will modify the atomic
               let old = counter1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
               if old == 5 {
                  ctrlc_sender1
                     .send(())
                     .expect("Must be possible to send the message");
               }
               // and return a reading:
               sensors::Reading {
                  measurement: Ok(old as sensors::TempType),
                  id,
               }
            })),
         )) as Box<dyn sensors::Sensor + std::marker::Send>
      });

      const SENSOR_NAME1: &str = "Sensor:BottomTube";

      let factories: std::collections::HashMap<String, SensorFactory> =
         std::collections::HashMap::from([(String::from(SENSOR_NAME1), factory1)]);

      let mut sink = sink::FakeSink::default();

      // -----------------------------------------------------------------------------------------------------
      // Run test:

      run(
         factories,
         &mut sink,
         ctrlc_receiver,
         std::time::Duration::from_millis(1),
      );

      // -----------------------------------------------------------------------------------------------------
      // Check results:
      // println!("sink: {sink:?}");
      assert!(!sink.items.is_empty());
      for (i, item) in sink.items.iter().enumerate() {
         assert!(item.NameToTemp.contains_key(SENSOR_NAME1));
         assert_eq!(
            *item.NameToTemp.get(SENSOR_NAME1).unwrap(),
            i as sensors::TempType
         );
         assert!(item.ErrorString.is_empty());
      }
   }

   #[test]
   fn two_sensors_check_data_provided_by_sensor_is_published() {
      // -----------------------------------------------------------------------------------------------------
      // Setup

      let (ctrlc_sender, ctrlc_receiver): (channel::Sender<()>, channel::Receiver<()>) =
         channel::bounded(100);
      // let mut counter = std::sync::Arc::new(std::sync::atomic::AtomicI32::default());
      // let mut counter1 = counter.clone();
      // let mut counter2 = counter.clone();

      let mut ctrlc_sender1 = ctrlc_sender.clone();
      let mut ctrlc_sender2 = ctrlc_sender.clone();

      // let mut counter1 = std::sync::Arc::<i32>::default();
      let mut counter1 = -1;
      let mut counter2 = -1;

      let factory1: crate::sensors_poller::SensorFactory = Box::new(move |id| {
         Box::new(sensors::MockSensor::new(
            id,
            std::cell::RefCell::new(Box::new(move || {
               counter1 += 1;
               if counter1 == 5 {
                  ctrlc_sender1
                     .send(())
                     .expect("Must be possible to send the message");
               }
               sensors::Reading {
                  measurement: Ok(counter1 as sensors::TempType),
                  id,
               }
            })),
         )) as Box<dyn sensors::Sensor + std::marker::Send>
      });
      let factory2: crate::sensors_poller::SensorFactory = Box::new(move |id| {
         Box::new(sensors::MockSensor::new(
            id,
            std::cell::RefCell::new(Box::new(move || {
               counter2 += 1;
               if counter2 == 5 {
                  ctrlc_sender2
                     .send(())
                     .expect("Must be possible to send the message");
               }
               sensors::Reading {
                  measurement: Ok(counter2 as sensors::TempType),
                  id,
               }
            })),
         )) as Box<dyn sensors::Sensor + std::marker::Send>
      });
      const SENSOR_NAME1: &str = "Sensor:BottomTube";
      const SENSOR_NAME2: &str = "Sensor:Ambient";

      let factories: std::collections::HashMap<String, SensorFactory> =
         std::collections::HashMap::from([
            (String::from(SENSOR_NAME1), factory1),
            (String::from(SENSOR_NAME2), factory2),
         ]);

      let mut sink = sink::FakeSink::default();

      // -----------------------------------------------------------------------------------------------------
      // Run test:

      run(
         factories,
         &mut sink,
         ctrlc_receiver,
         std::time::Duration::from_millis(1),
      );

      // -----------------------------------------------------------------------------------------------------
      // Check results:
      // println!("sink: {sink:?}");
      assert!(!sink.items.is_empty());
      for (i, item) in sink.items.iter().enumerate() {
         assert!(item.NameToTemp.contains_key(SENSOR_NAME1));
         assert!(item.NameToTemp.contains_key(SENSOR_NAME2));
         assert_eq!(
            *item.NameToTemp.get(SENSOR_NAME1).unwrap(),
            i as sensors::TempType
         );
         assert_eq!(
            *item.NameToTemp.get(SENSOR_NAME2).unwrap(),
            i as sensors::TempType
         );
         assert!(item.ErrorString.is_empty());
      }
   }

   #[test]
   fn two_sensors_one_of_them_is_slow_check_error_is_reported() {}
}
