#[allow(non_snake_case)]
mod DS18B20;

// #[derive(PartialEq, Debug)]
// struct ErrorDesc(String);
// type Result<T> = std::result::Result<T, ErrorDesc>;
// impl ErrorDesc {
//    pub fn new_err<T>(function_name: &str, desc: &str) -> Result<T> {
//       Err(ErrorDesc(format!("Failed to {function_name}: {desc}")))
//    }
// }

// impl<T: std::fmt::Debug> From<T> for ErrorDesc {
//    fn from(value: T) -> Self {
//       // ErrorDesc(TODO: print value in a string...);
//     }
// }

pub fn add(left: usize, right: usize) -> usize {
   left + right
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn it_works() {
      let result = add(2, 2);
      assert_eq!(result, 4);
   }
}
