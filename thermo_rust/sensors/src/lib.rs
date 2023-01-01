use anyhow::Result;

#[allow(non_snake_case)]
mod DS18B20;

pub trait Sensor {
   fn new(id: i32, path: &str) -> Self;
   fn id(&self) -> i32;
   fn path(&self) -> &str;
   fn read(&self) -> Result<f64>;
}

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   #[allow(non_snake_case)]
   pub mod Sensor {
      #[allow(unused_imports)]
      use super::super::*;

      pub fn id_can_be_fetched<T: Sensor>() {
         let sensor = T::new(1234, "asdf");
         assert_eq!(sensor.id(), 1234);
      }
      pub fn path_can_be_fetched<T: Sensor>() {
         let sensor = T::new(1234, "asdf");
         assert_eq!(sensor.path(), "asdf");
      }
   }

   #[test]
   fn it_works() {
      let result = 2 + 2;
      assert_eq!(result, 4);
   }
}
