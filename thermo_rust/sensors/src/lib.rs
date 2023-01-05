use anyhow::Result;

#[allow(non_snake_case)]
mod DS18B20;

pub trait Sensor {
   // fn new(id: i32, path: &str) -> Self;
   fn id(&self) -> i32;
   // fn path(&self) -> &str;
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
   }

   #[test]
   fn it_works() {
      let result = 2 + 2;
      assert_eq!(result, 4);
   }
}
