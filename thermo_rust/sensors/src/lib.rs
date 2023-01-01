#[allow(non_snake_case)]
mod DS18B20;

trait Sensor {
   fn new(id: i32, path: &str) -> Self;
   fn id(&self) -> i32;
   fn path(&self) -> &str;
   fn read(&self) -> f64;
}

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   #[test]
   fn it_works() {
      let result = 2 + 2;
      assert_eq!(result, 4);
   }
}
