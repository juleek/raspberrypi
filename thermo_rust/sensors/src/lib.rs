use anyhow::Result;

#[allow(non_snake_case)]
pub mod DS18B20;

pub type IdType = i32;
pub type TempType = f64;

#[derive(Debug)]
pub struct Reading {
   pub measurement: Result<TempType>,
   pub id: IdType,
}

pub trait Sensor {
   fn id(&self) -> IdType;
   fn read(&self) -> Reading;
}

pub struct FakeSensor {
   temperature: TempType,
   id: IdType,
}
impl FakeSensor {
   pub fn new(id: IdType, temperature: TempType) -> Self {
      FakeSensor { id, temperature }
   }
}
impl Sensor for FakeSensor {
   fn id(&self) -> IdType {
      self.id
   }
   fn read(&self) -> Reading {
      std::thread::sleep(std::time::Duration::from_millis(250));
      Reading {
         measurement: Ok(self.temperature),
         id: self.id,
      }
   }
}

pub struct MockSensor {
   id: IdType,
   read: std::cell::RefCell<Box<dyn FnMut() -> Reading + Send + Sync>>,
}
impl MockSensor {
   pub fn new(
      id: IdType,
      read: std::cell::RefCell<Box<dyn FnMut() -> Reading + Send + Sync>>,
   ) -> Self {
      MockSensor { id, read }
   }
}
impl Sensor for MockSensor {
   fn id(&self) -> IdType {
      self.id
   }
   fn read(&self) -> Reading {
      (self.read.borrow_mut())()
   }
}

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   #[allow(non_snake_case)]
   pub mod Sensor {
      #[allow(unused_imports)]
      use super::super::*;
   }

   #[test]
   fn it_works() {
      let result = 2 + 2;
      assert_eq!(result, 4);
   }
}
