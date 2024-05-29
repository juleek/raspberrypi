use anyhow::{anyhow, Context, Result};
use std::io::Read;

fn parse(data: &str) -> Result<f64> {
   const MAX: usize = 2 * 1024;
   let second_line = {
      let data = &data[..std::cmp::min(MAX, data.len())];
      let start = data.find('\n')
                      .with_context(|| anyhow!("Failed to find start of second line in: {}", &data))?;
      let data = &data[start + 1..];
      let end = data.find('\n').unwrap_or(data.len());
      &data[..end]
   };
   let temperature = {
      let start = &second_line.find("t=").with_context(|| anyhow!("Failed to find t= in: {second_line}"))?;
      let temp = &second_line[start + 2..];
      let temp: i32 =
         temp.parse().with_context(|| anyhow!("Failed to parse temperature as integer: {temp}"))?;
      temp as f64 / 1000.0
   };
   Ok(temperature)
}


#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

   #[derive(Debug)]
   struct OkParseTC {
      data:     String,
      expected: f64,
   }

   #[test]
   fn test_parse_returns_ok() {
      let test_cases = [
         //
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=18375"), expected: 18.375, },      // normal
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=0375"), expected: 0.375, },        // leading zeros 0100 => 0.1
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=-18375"), expected: -18.375, },    // negative
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=18375\n 26"), expected: 18.375, }, // more than 2 lines, 2nd line has correct data
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
         ErrorParseTC { data: String::new() },                                 // empty
         ErrorParseTC { data: String::from("26: crc=64 YES"), },               // one line
         ErrorParseTC { data: String::from("26: crc=64 YES t=1\n18325"), },    // t= in 1st line only
         ErrorParseTC { data: String::from("26: crc=64 YES\n 26 t=375ABC"), }, // letters after number
         ErrorParseTC { data: String::from("26: crc=64 YES\n 26 t=ABC375"), }, // letters before number
         ErrorParseTC { data: String::from("26: crc=64 YES\n 26 t=1 t=2"), },  // multiple t=
         //
      ];

      for tc in test_cases {
         let res = parse(&tc.data);
         println!("{res:?}");
         assert!(res.is_err());
      }
   }
}

// println!()
