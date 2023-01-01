use anyhow::{anyhow, Context, Result};
use std::io::BufRead;
use stdext::function_name;

// -----------------------------------------------------------------------------------------------------------

fn parse_from_str<T>(str: &str) -> Result<T>
where
   T: std::str::FromStr, // Something that can be parsed and created from a string
   T::Err: std::fmt::Debug, // And error of doing so should be printable
{
   match str.parse::<T>() {
      Ok(val) => Ok(val),
      Err(e) => Err(anyhow!("Failed to {} {str}: {e:?}", function_name!())),
   }
}

// -----------------------------------------------------------------------------------------------------------

fn parse_temperature_from_stream<'a, It, O, E>(it: It) -> Result<f64>
where
   It: std::iter::Iterator<Item = &'a std::result::Result<O, E>> + std::clone::Clone,
   E: 'a + std::fmt::Debug,
   O: 'a + std::convert::AsRef<str> + std::fmt::Debug,
{
   let mut copy: It = it.clone();
   let second_line: &Result<O, E> = match copy.nth(1) {
      Some(val) => val,
      None => {
         return Err(anyhow!(
            "Number of strings <= 1: {:?}",
            it.collect::<Vec<&std::result::Result<O, E>>>()
         ))
      }
   };
   let second_line: &O = match second_line {
      Ok(val) => val,
      Err(why) => return Err(anyhow!("Failed to read second line: {:?}", why)),
   };
   let second_line: &str = second_line.as_ref();

   static PATTERN: &str = " t=";
   let pos = match second_line.rfind(PATTERN) {
      Some(val) => val,
      None => {
         return Err(anyhow!(
            "The \"{PATTERN}\" pattern has not been found in the second_line: {second_line}"
         ))
      }
   };
   // println!("Second line: {:?}", &second_line[pos+PATTERN.len()..]);
   let temperature: i64 = parse_from_str(&second_line[pos + PATTERN.len()..])
      .with_context(|| format!("Failed to parse temperature from {second_line}"))?;
   Ok(temperature as f64 / 1000.)
}

// -----------------------------------------------------------------------------------------------------------

// fn parse_temperature_from_file(path: &std::path::Path) -> Result<f64> {
//    let file: std::fs::File = match std::fs::File::open(path) {
//       Ok(val) => val,
//       Err(why) => return Err(anyhow!("Failed to {} {path:?}: {why}", function_name!())),
//    };
//    let reader = std::io::BufReader::new(file);
//    parse_temperature_from_stream(reader.lines())
// }

// -----------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;
//    #[test]
//    fn test_conv_conv() {
//       let vec = vec![
//          Ok::<std::string::String, std::convert::Infallible>(String::from("asdf")),
//          Ok::<std::string::String, std::convert::Infallible>(String::from("qwer")),
//          Ok::<std::string::String, std::convert::Infallible>(String::from("zxcv")),
//       ];
//       let mut copy = vec.iter().clone();
//       let opt = copy.nth(1);
//       let unwrapped = copy.nth(1).unwrap();
//       let second_line = match copy.nth(1) {
//          Some(val) => val,
//          None => return,
//       };
//    }

   mod parse_from_str {
      use super::super::*;
      #[test]
      fn string_can_be_parsed_as_string() -> Result<()> {
         let actual: String = parse_from_str("asdf")?;
         assert_eq!(actual, String::from("asdf"));
         Ok(())
      }
      #[test]
      fn string_cannot_be_parsed_as_int() {
         let actual: Result<i32> = parse_from_str("asdf");
         assert!(actual.is_err());
         // println!("{:?}", actual.err().unwrap());
      }

      #[test]
      fn number_can_be_parsed_as_float() -> Result<()> {
         let actual: f32 = parse_from_str("12.359")?;
         assert_eq!(actual, 12.359);
         Ok(())
      }

      #[test]
      fn number_can_be_parsed_as_int() -> Result<()> {
         let actual: i32 = parse_from_str("12")?;
         assert_eq!(actual, 12);
         Ok(())
      }
   }

   mod parse_temperature_from_stream {
      use super::super::*;

        #[test]
        fn one_line_results_in_error() -> Result<()> {
           let v: Vec<std::result::Result<&str, std::convert::Infallible>> =
              vec![Ok("12 01 4b 46 7f ff 0e 10 b6 t=17125")];
           let actual = parse_temperature_from_stream(v.iter());
           assert!(actual.is_err());
           Ok(())
        }

        #[test]
        fn valid_input_returns_expected_temperature() -> Result<()> {
           let VALID_INPUT: Vec<std::result::Result<&str, std::convert::Infallible>> = vec![
              Ok("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES"),
              Ok("12 01 4b 46 7f ff 0e 10 b6 t=17125"),
           ];
           let actual = parse_temperature_from_stream(VALID_INPUT.iter())?;
           assert_eq!(actual, 17.125);
           println!("valid actual: {:?}", actual);
           Ok(())
        }
        #[test]
        fn bad_integer_temperature_returns_error() -> Result<()> {
           let VALID_INPUT: Vec<std::result::Result<&str, std::convert::Infallible>> = vec![
              Ok("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES"),
              Ok("12 01 4b 46 7f ff 0e 10 b6 t=17sss125"),
           ];
           let actual = parse_temperature_from_stream(VALID_INPUT.iter());
           assert!(actual.is_err());
           // println!("valid actual: {:?}", actual);
           Ok(())
        }

        #[test]
        fn works_with_strs() -> Result<()> {
           let v: Vec<std::result::Result<&str, std::convert::Infallible>> = vec![
              Ok("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES"),
              Ok("12 01 4b 46 7f ff 0e 10 b6 t=17125"),
           ];
           parse_temperature_from_stream(v.iter());
           Ok(())
        }
        #[test]
        fn works_with_strings() -> Result<()> {
           let v: Vec<std::result::Result<String, std::convert::Infallible>> = vec![
              Ok(String::from("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES")),
              Ok(String::from("12 01 4b 46 7f ff 0e 10 b6 t=17125")),
           ];
           let t = match v.iter().next().unwrap() {
              Ok(val) => val,
              Err(_) => return Err(anyhow!("")),
           };
           parse_temperature_from_stream(v.iter());
           Ok(())
        }
   }
}
