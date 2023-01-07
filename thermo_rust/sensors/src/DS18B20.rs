use anyhow::{anyhow, Context, Result};
use std::io::BufRead;
use stdext::function_name;

use crate::IdType;
use crate::Reading;
use crate::TempType;

// -----------------------------------------------------------------------------------------------------------

#[allow(dead_code)]
fn parse_from_str<T>(str: &str) -> Result<T>
where
   T: std::str::FromStr, // Something that can be parsed and created from a string
   T::Err: std::fmt::Debug, // And error of doing so should be printable
{
   str.parse::<T>()
      .map_err(|ref e| anyhow!("Failed to {} {str}: {e:?}", function_name!()))
}

// -----------------------------------------------------------------------------------------------------------

#[allow(dead_code)]
pub fn parse_temperature_from_stream<'a, It, O, E>(it: It) -> Reading
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
   let second_line: &O = vec[1].as_ref().map_err(|why| {
      anyhow!(
         "Failed to {}: Failed to read second line: {why:?}",
         function_name!()
      )
   })?;

   let second_line: &str = second_line.as_ref();

   const PATTERN: &str = " t=";
   let pos = second_line.rfind(PATTERN).ok_or_else(|| anyhow!(
      "Failed to {}: the \"{PATTERN}\" pattern has not been found in the second_line: {second_line}", function_name!()
   ))?;

   // println!("Second line: {:?}", &second_line[pos+PATTERN.len()..]);
   let temperature: i64 = parse_from_str(&second_line[pos + PATTERN.len()..])
      .with_context(|| format!("Failed to {}", function_name!()))?;
   Ok(temperature as TempType / 1000.)
}

// -----------------------------------------------------------------------------------------------------------

#[allow(dead_code)]
pub fn parse_temperature_from_file(path: &std::path::Path) -> Reading {
   let file: std::fs::File = std::fs::File::open(path)
      .map_err(|ref why| anyhow!("Failed to {} {path:?}: {why}", function_name!()))?;

   let reader = std::io::BufReader::new(file);
   parse_temperature_from_stream(reader.lines())
      .with_context(|| format!("Failed to {}: {path:?}", function_name!()))
}

#[derive(Debug)]
pub struct Sensor {
   path: std::path::PathBuf,
   id: IdType,
}

impl Sensor {
   pub fn new(id: IdType, path: std::path::PathBuf) -> Self {
      Sensor { path, id }
   }
   pub fn path(&self) -> &std::path::PathBuf {
      &self.path
   }
}
impl crate::Sensor for Sensor {
   fn id(&self) -> IdType {
      self.id
   }
   fn read(&self) -> Reading {
      parse_temperature_from_file(std::path::Path::new(&self.path))
   }
}

// ===========================================================================================================
// tests

#[cfg(test)]
mod tests {
   #[allow(unused_imports)]
   use super::*;

   mod Sensor {
      #[allow(unused_imports)]
      use super::super::*;

      #[test]
      fn id_can_be_fetched() {
         let sensor = Sensor::new(1234, std::path::PathBuf::from("asdf"));
         assert_eq!(<Sensor as crate::Sensor>::id(&sensor), 1234);
      }
      #[test]
      fn path_can_be_fetched() {
         let sensor = Sensor::new(1234, std::path::PathBuf::from("asdf"));
         assert_eq!(sensor.path(), &std::path::PathBuf::from("asdf"));
      }
   }

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
