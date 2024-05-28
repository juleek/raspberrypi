use anyhow::{anyhow, Context, Result};
use std::io::Read;

fn parse(sensor_info: &str) -> Result<f64> {
   let lines: Vec<&str> = sensor_info.lines().collect();
   if lines.is_empty() || lines.len() == 1 {
      return Err(anyhow!("Failed to parse lines. Lines: {lines:?}"));
   }
   let start_index = lines[1].find("t=");
   if start_index == None {
      return Err(anyhow!("Failed to find start_index in line {:?}", lines[1]));
   }
   let temp_str = &lines[1][start_index + 2..];
   let temp_float: Result<f64> = temp_str.parse();
   assert!(temp_float.is_ok());
   let temp = temp_float.unwrap() / 1000.0;
   Ok(temp)
}


#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

   #[derive(Debug)]
   struct OkParseTC {
      data:   String,
      expected: f64,
   }

   #[test]
   fn test_parse_returns_ok() {
      let test_cases = [
         //
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=18375"), expected: 18.375, },
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=375"), expected: 0.375, },


         // normal
         // leading zeros 0100 => 0.1
         // negative
         // more than 2 lines, 2nd line has correct data

         //
         ];

      for (i, tc) in test_cases.iter().enumerate() {
         let res = parse(&tc.data);
         assert!(res.is_ok());
         assert_eq!(res.unwrap(), tc.expected, "Test-case #{i}: {tc:?}");
      }
   }

   struct ErrorParseTC {
      data: String,
   }

   #[test]
   fn test_parse_returns_error() {
      let test_cases = [
         //
         ErrorParseTC { data: String::new() },
         ErrorParseTC { data: String::from("26: crc=64 YES"), },
         //


         // empty
         // one line
         // t= in 1st line only
         // letters after number
         // letters before number
         // multiple t=
         ];

      for tc in test_cases {
         let res = parse(&tc.data);
         println!("{res:?}");
         assert!(res.is_err());
      }
   }
}

// println!()
