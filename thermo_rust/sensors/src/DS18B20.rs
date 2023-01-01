use anyhow::{anyhow, Context, Result};
use std::io::BufRead;
use stdext::function_name;

// -----------------------------------------------------------------------------------------------------------

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn parse_temperature_from_stream<'a, It, O, E>(it: It) -> Result<f64>
where
   // We require Item to be owning (non-ref), because in the primary use-case, when we read strings from a
   // file, iterator yields io::Result<String> (not &io::Result<String>, not io::Result<&String>)
   // (which is not surpsing, because Rust's IO does not store entire content of a file into memory)
   It: std::iter::Iterator<Item = std::result::Result<O, E>>,
   E: 'a + std::fmt::Debug,
   O: 'a + std::convert::AsRef<str> + std::fmt::Debug,
{
   let vec: Vec<Result<O, E>> = it.take(10).collect();
   if vec.len() != 2 {
      return Err(anyhow!(
         "Failed to {}: Number of strings != 2: first 10 of them: {vec:?}",
         function_name!()
      ));
   }
   let second_line: &O = match &vec[1] {
      Ok(val) => val,
      Err(why) => {
         return Err(anyhow!(
            "Failed to {}: Failed to read second line: {why:?}",
            function_name!()
         ))
      }
   };
   let second_line: &str = second_line.as_ref();

   static PATTERN: &str = " t=";
   let pos = match second_line.rfind(PATTERN) {
      Some(val) => val,
      None => {
         return Err(anyhow!(
            "Failed to {}: the \"{PATTERN}\" pattern has not been found in the second_line: {second_line}", function_name!()
         ))
      }
   };
   // println!("Second line: {:?}", &second_line[pos+PATTERN.len()..]);
   let temperature: i64 = parse_from_str(&second_line[pos + PATTERN.len()..])
      .with_context(|| format!("Failed to {}", function_name!()))?;
   Ok(temperature as f64 / 1000.)
}

// -----------------------------------------------------------------------------------------------------------

#[allow(dead_code)]
pub fn parse_temperature_from_file(path: &std::path::Path) -> Result<f64> {
   let file: std::fs::File = match std::fs::File::open(path) {
      Ok(val) => val,
      Err(why) => return Err(anyhow!("Failed to {} {path:?}: {why}", function_name!())),
   };
   let reader = std::io::BufReader::new(file);
   parse_temperature_from_stream(reader.lines())
      .with_context(|| format!("Failed to {}: {path:?}", function_name!()))
}

// ===========================================================================================================
// tests

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   mod parse_temperature_from_file {
      use super::super::*;
      #[test]
      #[ignore = "is not a unit-test, requires a file"]
      fn test_read_from_file() -> Result<()> {
         let _actual = parse_temperature_from_file(std::path::Path::new("/home/dimanne/test.txt"));
         // println!("{actual:?}");
         Ok(())
      }
   }

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
         let actual = parse_temperature_from_stream(v.into_iter());
         assert!(actual.is_err());
         Ok(())
      }

      #[test]
      fn valid_input_returns_expected_temperature() -> Result<()> {
         let VALID_INPUT: Vec<std::result::Result<&str, std::convert::Infallible>> = vec![
            Ok("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES"),
            Ok("12 01 4b 46 7f ff 0e 10 b6 t=17125"),
         ];
         let actual = parse_temperature_from_stream(VALID_INPUT.into_iter())?;
         assert_eq!(actual, 17.125);
         // println!("valid actual: {:?}", actual);
         Ok(())
      }
      #[test]
      fn bad_integer_temperature_returns_error() -> Result<()> {
         let VALID_INPUT: Vec<std::result::Result<&str, std::convert::Infallible>> = vec![
            Ok("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES"),
            Ok("12 01 4b 46 7f ff 0e 10 b6 t=17sss125"),
         ];
         let actual = parse_temperature_from_stream(VALID_INPUT.into_iter());
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
         #[allow(unused_must_use)]
         let _ = parse_temperature_from_stream(v.into_iter());
         Ok(())
      }
      #[test]
      fn works_with_strings() -> Result<()> {
         let v: Vec<std::result::Result<String, std::convert::Infallible>> = vec![
            Ok(String::from("12 01 4b 46 7f ff 0e 10 b6 : crc=b6 YES")),
            Ok(String::from("12 01 4b 46 7f ff 0e 10 b6 t=17125")),
         ];
         let _ = parse_temperature_from_stream(v.into_iter());
         Ok(())
      }
   }
}
